#include "runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void bunzo_print_int(int64_t val) {
    printf("%lld\n", (long long)val);
}

void bunzo_print_float(double val) {
    printf("%g\n", val);
}

void bunzo_print_string(const char* val) {
    printf("%s\n", val ? val : "null");
}

void bunzo_print_bool(bool val) {
    printf("%s\n", val ? "true" : "false");
}

void bunzo_print_null(void) {
    printf("null\n");
}

const char* bunzo_concat_strings(const char* l, const char* r) {
    if (!l) l = "";
    if (!r) r = "";
    size_t len_l = strlen(l);
    size_t len_r = strlen(r);
    char* res = bunzo_gc_malloc(len_l + len_r + 1);
    strcpy(res, l);
    strcat(res, r);
    return res;
}

bool bunzo_equal_strings(const char* l, const char* r) {
    if (l == r) return true;
    if (!l || !r) return false;
    return strcmp(l, r) == 0;
}

bool bunzo_notequal_strings(const char* l, const char* r) {
    return !bunzo_equal_strings(l, r);
}

// ── Memory Management / Garbage Collector Implementation ──────────────────

typedef struct GCAllocation {
    void* ptr;
    size_t size;
    bool marked;
    struct GCAllocation* next;
} GCAllocation;

typedef struct GCRoot {
    void** ptr;
    struct GCRoot* next;
} GCRoot;

GCAllocation* bunzo_gc_allocations = NULL;
GCRoot* bunzo_gc_roots = NULL;
static void* stack_bottom = NULL;

void bunzo_gc_init(void* sb) {
    stack_bottom = sb;
}

void bunzo_gc_register_root(void** ptr) {
    GCRoot* root = malloc(sizeof(GCRoot));
    if (!root) {
        fprintf(stderr, "Out of memory in GC root registration\n");
        exit(1);
    }
    root->ptr = ptr;
    root->next = bunzo_gc_roots;
    bunzo_gc_roots = root;
}

static size_t gc_alloc_count(void) {
    size_t count = 0;
    GCAllocation* curr = bunzo_gc_allocations;
    while (curr) {
        count++;
        curr = curr->next;
    }
    return count;
}

void* bunzo_gc_malloc(size_t size) {
    static size_t bytes_allocated_since_last_gc = 0;

    // Trigger GC if we have allocated more than 1MB since last run or have many allocations
    if (bytes_allocated_since_last_gc > 1024 * 1024 || gc_alloc_count() > 500) {
        bunzo_gc_collect();
        bytes_allocated_since_last_gc = 0;
    }

    void* ptr = malloc(size);
    if (!ptr) {
        bunzo_gc_collect();
        ptr = malloc(size);
        if (!ptr) {
            fprintf(stderr, "Out of memory\n");
            exit(1);
        }
    }
    memset(ptr, 0, size);

    GCAllocation* alloc = malloc(sizeof(GCAllocation));
    if (!alloc) {
        free(ptr);
        fprintf(stderr, "Out of memory in GC allocator metadata\n");
        exit(1);
    }
    alloc->ptr = ptr;
    alloc->size = size;
    alloc->marked = false;
    alloc->next = bunzo_gc_allocations;
    bunzo_gc_allocations = alloc;

    bytes_allocated_since_last_gc += size;
    return ptr;
}

static void gc_mark_block(void* ptr) {
    GCAllocation* curr = bunzo_gc_allocations;
    while (curr) {
        if (!curr->marked && curr->ptr == ptr) {
            curr->marked = true;
            // Recursive scan for interior pointers
            size_t words = curr->size / sizeof(void*);
            void** block_ptr = (void**)curr->ptr;
            for (size_t i = 0; i < words; ++i) {
                gc_mark_block(block_ptr[i]);
            }
            break;
        }
        curr = curr->next;
    }
}

void bunzo_gc_collect(void) {
    if (!stack_bottom) return;

    volatile void* local_var = NULL;
    void** stack_top = (void**)&local_var;

    // Unmark all allocations
    GCAllocation* curr = bunzo_gc_allocations;
    while (curr) {
        curr->marked = false;
        curr = curr->next;
    }

    // Scan stack
    void** low = (void**)stack_bottom;
    void** high = (void**)stack_top;
    if (low > high) {
        void** temp = low;
        low = high;
        high = temp;
    }

    for (void** p = low; p < high; ++p) {
        gc_mark_block(*p);
    }

    // Scan registered roots
    GCRoot* root = bunzo_gc_roots;
    while (root) {
        if (root->ptr && *(root->ptr)) {
            gc_mark_block(*(root->ptr));
        }
        root = root->next;
    }

    // Sweep phase
    GCAllocation* prev = NULL;
    curr = bunzo_gc_allocations;
    while (curr) {
        if (!curr->marked) {
            free(curr->ptr);
            GCAllocation* to_free = curr;
            curr = curr->next;
            if (prev) {
                prev->next = curr;
            } else {
                bunzo_gc_allocations = curr;
            }
            free(to_free);
        } else {
            prev = curr;
            curr = curr->next;
        }
    }
}

void bunzo_gc_cleanup(void) {
    // Free all remaining allocations
    GCAllocation* curr = bunzo_gc_allocations;
    while (curr) {
        free(curr->ptr);
        GCAllocation* next = curr->next;
        free(curr);
        curr = next;
    }
    bunzo_gc_allocations = NULL;

    // Free all registered roots
    GCRoot* root = bunzo_gc_roots;
    while (root) {
        GCRoot* next = root->next;
        free(root);
        root = next;
    }
    bunzo_gc_roots = NULL;
}
