#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>

#if defined(_WIN32)
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <io.h>
#include <process.h>
#include <windows.h>
#else
#include <sys/wait.h>
#include <unistd.h>
#endif

struct buffer {
  char *data;
  size_t len;
  size_t cap;
};

struct symbol_entry {
  const char *symbol;
  size_t symbol_len;
  const char *object;
  size_t object_len;
  char type;
};

struct symbol_list {
  struct symbol_entry *entries;
  size_t len;
  size_t cap;
};

static void print_errno(const char *what) {
  fprintf(stderr, "static_library_validator: %s: %s\n", what, strerror(errno));
}

static void *xrealloc(void *ptr, size_t size) {
  void *new_ptr = realloc(ptr, size);
  if (new_ptr == NULL) {
    fprintf(stderr, "static_library_validator: out of memory\n");
    exit(2);
  }
  return new_ptr;
}

static void buffer_init(struct buffer *buf) {
  buf->data = NULL;
  buf->len = 0;
  buf->cap = 0;
}

static void buffer_reserve(struct buffer *buf, size_t extra) {
  size_t need = buf->len + extra + 1;
  if (need <= buf->cap) {
    return;
  }
  size_t cap = buf->cap == 0 ? 4096 : buf->cap;
  while (cap < need) {
    cap *= 2;
  }
  buf->data = xrealloc(buf->data, cap);
  buf->cap = cap;
}

static void buffer_append(struct buffer *buf, const char *data, size_t len) {
  buffer_reserve(buf, len);
  memcpy(buf->data + buf->len, data, len);
  buf->len += len;
  buf->data[buf->len] = '\0';
}

static int read_file(FILE *file, struct buffer *buf) {
  char chunk[4096];

  rewind(file);
  while (1) {
    size_t n = fread(chunk, 1, sizeof(chunk), file);
    if (n != 0) buffer_append(buf, chunk, n);
    if (n != sizeof(chunk)) {
      if (ferror(file)) {
        print_errno("fread");
        return 0;
      }
      return 1;
    }
  }
}

static int copy_file(FILE *from, FILE *to) {
  char chunk[4096];

  rewind(from);
  while (1) {
    size_t n = fread(chunk, 1, sizeof(chunk), from);
    if (n != 0 && fwrite(chunk, 1, n, to) != n) {
      print_errno("fwrite");
      return 0;
    }
    if (n != sizeof(chunk)) {
      if (ferror(from)) {
        print_errno("fread");
        return 0;
      }
      return 1;
    }
  }
}

static void add_symbol(struct symbol_list *list, const char *symbol,
                       size_t symbol_len, const char *object,
                       size_t object_len, char type) {
  if (list->len == list->cap) {
    size_t cap = list->cap == 0 ? 64 : list->cap * 2;
    list->entries = xrealloc(list->entries, cap * sizeof(*list->entries));
    list->cap = cap;
  }
  list->entries[list->len].symbol = symbol;
  list->entries[list->len].symbol_len = symbol_len;
  list->entries[list->len].object = object;
  list->entries[list->len].object_len = object_len;
  list->entries[list->len].type = type;
  list->len++;
}

static int should_keep_type(char type) {
  return isupper((unsigned char)type) &&
         type != 'U' && type != 'V' && type != 'W' && type != 'C';
}

#if defined(_WIN32)

static wchar_t *utf8_to_wide(const char *s) {
  int len;
  wchar_t *wide;

  if (s[0] == '\0') {
    wide = xrealloc(NULL, sizeof(wchar_t));
    wide[0] = L'\0';
    return wide;
  }
  len = MultiByteToWideChar(CP_UTF8, 0, s, -1, NULL, 0);
  if (len <= 0) {
    return NULL;
  }
  wide = xrealloc(NULL, (size_t)len * sizeof(wchar_t));
  if (MultiByteToWideChar(CP_UTF8, 0, s, -1, wide, len) <= 0) {
    free(wide);
    return NULL;
  }
  return wide;
}

static int run(const char *const argv[], FILE *stdin_file, FILE *stdout_file) {
  int saved_in, saved_out;
  size_t argc = 0;
  wchar_t **wargv;
  intptr_t child;
  int status;

  saved_in = _dup(_fileno(stdin));
  saved_out = _dup(_fileno(stdout));
  if (saved_in < 0 || saved_out < 0) {
    print_errno("_dup");
    if (saved_in >= 0) _close(saved_in);
    if (saved_out >= 0) _close(saved_out);
    return 0;
  }

  if ((stdin_file != NULL &&
       _dup2(_fileno(stdin_file), _fileno(stdin)) != 0) ||
      (stdout_file != NULL &&
       _dup2(_fileno(stdout_file), _fileno(stdout)) != 0)) {
    print_errno("_dup2");
    _dup2(saved_in, _fileno(stdin));
    _dup2(saved_out, _fileno(stdout));
    _close(saved_in);
    _close(saved_out);
    return 0;
  }

  while (argv[argc] != NULL) {
    argc++;
  }
  wargv = xrealloc(NULL, (argc + 1) * sizeof(*wargv));
  for (size_t i = 0; i < argc; ++i) {
    wargv[i] = utf8_to_wide(argv[i]);
    if (wargv[i] == NULL) {
      fprintf(stderr, "static_library_validator: invalid UTF-8 argument\n");
      for (size_t j = 0; j < i; ++j) free(wargv[j]);
      free(wargv);
      return 0;
    }
  }
  wargv[argc] = NULL;

  child = _wspawnv(_P_NOWAIT, wargv[0], (const wchar_t *const *)wargv);
  for (size_t i = 0; i < argc; ++i) free(wargv[i]);
  free(wargv);

  _dup2(saved_in, _fileno(stdin));
  _dup2(saved_out, _fileno(stdout));
  _close(saved_in);
  _close(saved_out);

  if (child == -1) {
    print_errno("_wspawnv");
    return 0;
  }

  if (_cwait(&status, child, 0) == -1) {
    print_errno("_cwait");
    return 0;
  }
  if (status != 0) {
    fprintf(stderr,
            "static_library_validator: command exited with status %d\n",
            status);
    return 0;
  }
  return 1;
}

#else

static int run(const char *const argv[], FILE *stdin_file, FILE *stdout_file) {
  pid_t pid;
  int status;

  pid = fork();
  if (pid < 0) {
    print_errno("fork");
    return 0;
  }

  if (pid == 0) {
    if (stdin_file != NULL) dup2(fileno(stdin_file), STDIN_FILENO);
    if (stdout_file != NULL) dup2(fileno(stdout_file), STDOUT_FILENO);
    execv(argv[0], (char *const *)argv);
    fprintf(stderr, "static_library_validator: execv(%s): %s\n",
            argv[0], strerror(errno));
    _exit(127);
  }

  if (waitpid(pid, &status, 0) < 0) {
    print_errno("waitpid");
    return 0;
  }
  if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
    fprintf(stderr,
            "static_library_validator: command exited with status %d\n",
            WIFEXITED(status) ? WEXITSTATUS(status) : -1);
    return 0;
  }
  return 1;
}

#endif

static void parse_nm_line(const char *line, size_t len,
                          struct symbol_list *out) {
  const char *end = line + len;
  const char *open = memchr(line, '[', len);
  const char *close;
  const char *p;
  const char *symbol;
  size_t symbol_len;
  char type;

  if (open == NULL) return;
  close = memchr(open + 1, ']', (size_t)(end - open - 1));
  if (close == NULL) return;

  /* llvm-nm -A -P emits "<archive>[<object>]: <symbol> <type> ...". */
  if (close + 2 >= end || close[1] != ':' || close[2] != ' ') return;
  p = close + 3;
  while (p < end && isspace((unsigned char)*p)) p++;
  if (p == end) return;
  symbol = p;
  while (p < end && !isspace((unsigned char)*p)) p++;
  symbol_len = (size_t)(p - symbol);
  while (p < end && isspace((unsigned char)*p)) p++;
  if (p == end || p + 1 == end || !isspace((unsigned char)p[1])) return;

  type = *p;
  if (!should_keep_type(type)) return;
  add_symbol(out, symbol, symbol_len, open + 1,
             (size_t)(close - open - 1), type);
}

static int compare_symbol_entries(const void *a, const void *b) {
  const struct symbol_entry *entry_a = a;
  const struct symbol_entry *entry_b = b;
  size_t min_len = entry_a->symbol_len < entry_b->symbol_len
                       ? entry_a->symbol_len
                       : entry_b->symbol_len;
  int cmp = memcmp(entry_a->symbol, entry_b->symbol, min_len);
  if (cmp != 0) return cmp;
  return entry_a->symbol_len < entry_b->symbol_len
             ? -1
             : entry_a->symbol_len != entry_b->symbol_len;
}

static int touch(const char *path) {
  FILE *f = fopen(path, "ab");
  if (f == NULL) {
    print_errno("fopen");
    return 0;
  }
  if (fclose(f) != 0) {
    print_errno("fclose");
    return 0;
  }
  return 1;
}

static const char *required_env(const char *env_name) {
  const char *from_env = getenv(env_name);
  if (from_env != NULL && from_env[0] != '\0') {
    return from_env;
  }
  fprintf(stderr, "static_library_validator: required env var %s is not set\n",
          env_name);
  exit(2);
}

int main(int argc, char **argv) {
  const char *nm_args[7];
  const char *filt_args[2];
  const char *darwin_target;
  size_t nm_argc = 0;
  struct buffer nm_output;
  struct buffer dup_block;
  struct symbol_list entries = {0};
  FILE *nm_file;
  FILE *dup_file;
  FILE *demangled_file;

  if (argc != 3) {
    fprintf(stderr,
            "usage: static_library_validator <static_library> <stamp_path>\n");
    return 2;
  }

  nm_args[nm_argc++] = required_env("LLVM_NM");
  nm_args[nm_argc++] = "-A";
  nm_args[nm_argc++] = "-g";
  nm_args[nm_argc++] = "-P";
  darwin_target = getenv("DARWIN_TARGET");
  if (darwin_target != NULL && strcmp(darwin_target, "1") == 0) {
    nm_args[nm_argc++] = "--no-weak";
  }
  nm_args[nm_argc++] = argv[1];
  nm_args[nm_argc] = NULL;

  nm_file = tmpfile();
  if (nm_file == NULL) return print_errno("tmpfile"), 2;
  if (!run(nm_args, NULL, nm_file)) return 2;

  buffer_init(&nm_output);
  if (!read_file(nm_file, &nm_output)) return 2;
  fclose(nm_file);

  size_t line_start = 0;
  for (size_t i = 0; i <= nm_output.len; ++i) {
    if (i == nm_output.len || nm_output.data[i] == '\n') {
      parse_nm_line(nm_output.data + line_start, i - line_start, &entries);
      line_start = i + 1;
    }
  }

  qsort(entries.entries, entries.len, sizeof(*entries.entries),
        compare_symbol_entries);

  buffer_init(&dup_block);
  for (size_t i = 0; i < entries.len;) {
    size_t j = i + 1;
    while (j < entries.len && entries.entries[i].symbol_len ==
                                  entries.entries[j].symbol_len &&
           memcmp(entries.entries[i].symbol, entries.entries[j].symbol,
                  entries.entries[i].symbol_len) == 0) {
      j++;
    }
    if (j - i >= 2) {
      for (size_t k = i; k < j; ++k) {
        char type[] = {entries.entries[k].type, ' '};
        buffer_append(&dup_block, entries.entries[k].object,
                      entries.entries[k].object_len);
        buffer_append(&dup_block, ": ", 2);
        buffer_append(&dup_block, type, sizeof(type));
        buffer_append(&dup_block, entries.entries[k].symbol,
                      entries.entries[k].symbol_len);
        buffer_append(&dup_block, "\n", 1);
      }
    }
    i = j;
  }

  if (dup_block.len == 0) return touch(argv[2]) ? 0 : 2;

  filt_args[0] = required_env("LLVM_CXXFILT");
  filt_args[1] = NULL;
  dup_file = tmpfile();
  if (dup_file == NULL) return print_errno("tmpfile"), 2;
  if (fwrite(dup_block.data, 1, dup_block.len, dup_file) != dup_block.len) {
    return print_errno("fwrite"), 2;
  }
  rewind(dup_file);
  demangled_file = tmpfile();
  if (demangled_file == NULL) return print_errno("tmpfile"), 2;
  if (!run(filt_args, dup_file, demangled_file)) return 2;

  fprintf(stderr, "Duplicate symbols found in %s:\n", argv[1]);
  if (!copy_file(demangled_file, stderr)) return 2;
  fclose(dup_file);
  fclose(demangled_file);
  return 1;
}
