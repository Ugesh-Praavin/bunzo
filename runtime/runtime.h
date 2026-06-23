#ifndef BUNZO_RUNTIME_H
#define BUNZO_RUNTIME_H

#include <stdint.h>
#include <stdbool.h>

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

#endif // BUNZO_RUNTIME_H
