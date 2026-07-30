#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#define Sleep(x) usleep((x) * 1000)
#define DWORD unsigned int
#endif

typedef struct {
    int64_t count;
    int64_t items[256];
} LinkList;

static int link_argc = 0;
static char** link_argv = NULL;
static void link_runtime_init(int argc, char** argv) { link_argc = argc; link_argv = argv; }
static int64_t link_args_len(void) { return (int64_t)link_argc; }
static const char* link_arg(int64_t index) { return (index >= 0 && index < link_argc) ? link_argv[index] : ""; }
static int64_t link_str_len(const char* value) { return value ? (int64_t)strlen(value) : 0; }
static bool link_str_eq(const char* left, const char* right) { return strcmp(left ? left : "", right ? right : "") == 0; }
static char* link_str_concat(const char* left, const char* right) {
    size_t left_len = strlen(left ? left : "");
    size_t right_len = strlen(right ? right : "");
    char* result = (char*)malloc(left_len + right_len + 1);
    if (!result) { fputs("Link runtime: out of memory\n", stderr); exit(1); }
    memcpy(result, left ? left : "", left_len);
    memcpy(result + left_len, right ? right : "", right_len + 1);
    return result;
}
static char* link_str_substring(const char* value, int64_t start, int64_t end) {
    int64_t length = link_str_len(value);
    if (start < 0) start = 0; if (end > length) end = length; if (end < start) end = start;
    size_t count = (size_t)(end - start); char* result = (char*)malloc(count + 1);
    if (!result) { fputs("Link runtime: out of memory\n", stderr); exit(1); }
    memcpy(result, (value ? value : "") + start, count); result[count] = '\0'; return result;
}
static int64_t link_str_char_code(const char* value, int64_t index) {
    if (!value || index < 0 || index >= (int64_t)strlen(value)) return -1;
    return (int64_t)(unsigned char)value[index];
}
static char* link_file_read(const char* path) {
    FILE* file = fopen(path, "rb"); if (!file) return NULL;
    if (fseek(file, 0, SEEK_END) != 0) { fclose(file); return NULL; }
    long size = ftell(file); if (size < 0) { fclose(file); return NULL; } rewind(file);
    char* result = (char*)malloc((size_t)size + 1); if (!result) { fclose(file); return NULL; }
    size_t read = fread(result, 1, (size_t)size, file); fclose(file); result[read] = '\0'; return result;
}
static int64_t link_file_write(const char* path, const char* content) {
    FILE* file = fopen(path, "wb"); if (!file) return 0;
    size_t length = strlen(content ? content : ""); size_t written = fwrite(content ? content : "", 1, length, file);
    return fclose(file) == 0 && written == length;
}

int64_t absolute(int64_t value);
int64_t minimum(int64_t a, int64_t b);
int64_t maximum(int64_t a, int64_t b);
int64_t clamp(int64_t value, int64_t lo, int64_t hi);
int64_t fib(int64_t n);
int64_t factorial(int64_t n);
int64_t ipow(int64_t base, int64_t exp);
bool is_even(int64_t n);
bool is_odd(int64_t n);
int64_t sum(int64_t from, int64_t to);
bool starts_with(const char* s, const char* prefix);
bool ends_with(const char* s, const char* suffix);
bool contains(const char* s, const char* needle);
const char* repeat(const char* s, int64_t n);
bool is_empty(const char* s);
bool is_blank(const char* s);
int main(int argc, char** argv);

int64_t absolute(int64_t value) {
    if ((value < 0LL)) {
        return -(value);
    }
    return value;
}

int64_t minimum(int64_t a, int64_t b) {
    if ((a < b)) {
        return a;
    }
    return b;
}

int64_t maximum(int64_t a, int64_t b) {
    if ((a > b)) {
        return a;
    }
    return b;
}

int64_t clamp(int64_t value, int64_t lo, int64_t hi) {
    return min(max(value, lo), hi);
}

int64_t fib(int64_t n) {
    if ((n <= 1LL)) {
        return n;
    }
    return (fib((n - 1LL)) + fib((n - 2LL)));
}

int64_t factorial(int64_t n) {
    if ((n <= 1LL)) {
        return 1LL;
    }
    return (n * factorial((n - 1LL)));
}

int64_t ipow(int64_t base, int64_t exp) {
    int64_t result = 1LL;
    int64_t i = 0LL;
    while ((i < exp)) {
        result = (result * base);
        i = (i + 1LL);
    }
    return result;
}

bool is_even(int64_t n) {
    return ((n % 2LL) == 0LL);
}

bool is_odd(int64_t n) {
    return !(is_even(n));
}

int64_t sum(int64_t from, int64_t to) {
    int64_t total = 0LL;
    for (int64_t i = from; i < to; i++) {
        total = (total + i);
    }
    return total;
}

bool starts_with(const char* s, const char* prefix) {
    if ((link_str_len(prefix) > link_str_len(s))) {
        return false;
    }
    return link_str_eq(link_str_substring(s, 0LL, link_str_len(prefix)), prefix);
}

bool ends_with(const char* s, const char* suffix) {
    int64_t slen = link_str_len(s);
    int64_t flen = link_str_len(suffix);
    if ((flen > slen)) {
        return false;
    }
    return link_str_eq(link_str_substring(s, (slen - flen), slen), suffix);
}

bool contains(const char* s, const char* needle) {
    int64_t slen = link_str_len(s);
    int64_t nlen = link_str_len(needle);
    if ((nlen == 0LL)) {
        return true;
    }
    if ((nlen > slen)) {
        return false;
    }
    int64_t i = 0LL;
    int64_t max = ((slen - nlen) + 1LL);
    while ((i < max)) {
        if (link_str_eq(link_str_substring(s, i, (i + nlen)), needle)) {
            return true;
        }
        i = (i + 1LL);
    }
    return false;
}

const char* repeat(const char* s, int64_t n) {
    const char* result = "";
    int64_t i = 0LL;
    while ((i < n)) {
        result = link_str_concat(result, s);
        i = (i + 1LL);
    }
    return result;
}

bool is_empty(const char* s) {
    return (link_str_len(s) == 0LL);
}

bool is_blank(const char* s) {
    int64_t len = link_str_len(s);
    int64_t i = 0LL;
    while ((i < len)) {
        int64_t ch = link_str_char_code(s, i);
        if (((((ch != 32LL) && (ch != 9LL)) && (ch != 10LL)) && (ch != 13LL))) {
            return false;
        }
        i = (i + 1LL);
    }
    return true;
}

int main(int argc, char** argv) {
    link_runtime_init(argc, argv);
    printf("%lld\n", (long long)(absolute(-5LL)));
    printf("%lld\n", (long long)(minimum(3LL, 7LL)));
    printf("%lld\n", (long long)(maximum(3LL, 7LL)));
    printf("%lld\n", (long long)(fib(10LL)));
    printf("%lld\n", (long long)(factorial(5LL)));
    printf("%lld\n", (long long)(ipow(2LL, 10LL)));
    if (starts_with("hello world", "hello")) {
        printf("%lld\n", (long long)(1LL));
    } else {
        printf("%lld\n", (long long)(0LL));
    }
    if (ends_with("hello world", "world")) {
        printf("%lld\n", (long long)(1LL));
    } else {
        printf("%lld\n", (long long)(0LL));
    }
    if (contains("hello world", "lo w")) {
        printf("%lld\n", (long long)(1LL));
    } else {
        printf("%lld\n", (long long)(0LL));
    }
    printf("%lld\n", (long long)(is_blank("   ")));
    printf("%lld\n", (long long)(is_blank(" x ")));
    return 0;
}

