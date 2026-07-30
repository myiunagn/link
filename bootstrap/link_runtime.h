#ifndef LINK_BOOTSTRAP_RUNTIME_H
#define LINK_BOOTSTRAP_RUNTIME_H

// Link bootstrap runtime — single canonical source (v2).
// The Stage 1 seed compiler embeds these functions in generated C output.

#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ── arena allocator ──────────────────────────────────────────────────────────

#define LINK_ARENA_SZ (256 * 1024)
static char link_arena[LINK_ARENA_SZ];
static size_t link_arena_pos = 0;

static void* link_arena_alloc(size_t sz) {
    if (link_arena_pos + sz > LINK_ARENA_SZ) return malloc(sz);
    void* p = link_arena + link_arena_pos;
    link_arena_pos += sz;
    return p;
}

// ── print ────────────────────────────────────────────────────────────────────

static void link_print_i64(int64_t value) {
    printf("%lld\n", (long long)value);
}

// ── string helpers ───────────────────────────────────────────────────────────

static int64_t link_str_len(const char* value) {
    return value ? (int64_t)strlen(value) : 0;
}

static bool link_str_eq(const char* left, const char* right) {
    return strcmp(left ? left : "", right ? right : "") == 0;
}

static char* link_str_concat(const char* left, const char* right) {
    size_t ll = strlen(left ? left : "");
    size_t rl = strlen(right ? right : "");
    char* result = (char*)link_arena_alloc(ll + rl + 1);
    if (!result) { fputs("out of memory\n", stderr); exit(1); }
    memcpy(result, left ? left : "", ll);
    memcpy(result + ll, right ? right : "", rl + 1);
    return result;
}

static char* link_str_substring(const char* value, int64_t start, int64_t end) {
    int64_t length = link_str_len(value);
    if (start < 0) start = 0; if (end > length) end = length; if (end < start) end = start;
    size_t count = (size_t)(end - start);
    char* result = (char*)link_arena_alloc(count + 1);
    if (!result) { fputs("out of memory\n", stderr); exit(1); }
    memcpy(result, (value ? value : "") + start, count);
    result[count] = '\0';
    return result;
}

static int64_t link_str_char_code(const char* value, int64_t index) {
    if (!value || index < 0 || index >= (int64_t)strlen(value)) return -1;
    return (int64_t)(unsigned char)value[index];
}

// ── args ─────────────────────────────────────────────────────────────────────

static int link_argc = 0;
static char** link_argv = NULL;

static int64_t link_args_len(void) { return (int64_t)link_argc; }

static const char* link_arg(int64_t index) {
    return (index >= 0 && index < link_argc) ? link_argv[index] : "";
}

// ── file I/O ─────────────────────────────────────────────────────────────────

static char* link_file_read(const char* path) {
    FILE* f = fopen(path, "rb");
    if (!f) { fprintf(stderr, "error: cannot open '%s'\n", path); exit(1); }
    fseek(f, 0, SEEK_END); long sz = ftell(f); rewind(f);
    if (sz < 0) { fclose(f); fputs("error: cannot read file\n", stderr); exit(1); }
    char* buf = (char*)link_arena_alloc((size_t)sz + 1);
    if (!buf) { fclose(f); fputs("error: out of memory\n", stderr); exit(1); }
    size_t rd = fread(buf, 1, (size_t)sz, f); fclose(f); buf[rd] = '\0'; return buf;
}

static int64_t link_file_write(const char* path, const char* content) {
    FILE* f = fopen(path, "wb"); if (!f) return 0;
    size_t len = strlen(content ? content : "");
    size_t wr = fwrite(content ? content : "", 1, len, f);
    return (fclose(f) == 0 && wr == len);
}

#endif
