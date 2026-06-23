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
    char* res = malloc(len_l + len_r + 1);
    if (!res) {
        fprintf(stderr, "Out of memory\n");
        exit(1);
    }
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
