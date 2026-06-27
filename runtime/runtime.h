#ifndef BUNZO_RUNTIME_H
#define BUNZO_RUNTIME_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Print functions
void bunzo_print_int(int64_t val);
void bunzo_print_float(double val);
void bunzo_print_string(const char* val);
void bunzo_print_bool(bool val);
void bunzo_print_null(void);

// String operations
const char* bunzo_concat_strings(const char* l, const char* r);
bool bunzo_equal_strings(const char* l, const char* r);
bool bunzo_notequal_strings(const char* l, const char* r);

// Memory Management / Garbage Collector
void bunzo_gc_init(void* stack_bottom);
void* bunzo_gc_malloc(size_t size);
void bunzo_gc_register_root(void** ptr);
void bunzo_gc_cleanup(void);
void bunzo_gc_collect(void);

#endif // BUNZO_RUNTIME_H
