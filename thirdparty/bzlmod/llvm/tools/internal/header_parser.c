#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
  const char *path = getenv("PARSE_HEADER");
  if (path == NULL || path[0] == '\0') {
    fprintf(stderr, "header_parser: required env var PARSE_HEADER is not set\n");
    exit(2);
  }

  int fd = open(path, O_WRONLY | O_CREAT, 0666);
  if (fd < 0) {
    fprintf(stderr, "header_parser: failed to touch %s: %s\n",
            path, strerror(errno));
    exit(2);
  }
  if (close(fd) != 0) {
    fprintf(stderr, "header_parser: failed to close =%s: %s\n",
            path, strerror(errno));
    exit(2);
  }

  const char *clang_path = getenv("LLVM_CLANGXX");
  if (clang_path == NULL || clang_path[0] == '\0') {
    fprintf(stderr, "header_parser: required env var LLVM_CLANGXX is not set\n");
    exit(2);
  }

  argv[0] = (char *)clang_path;
  execv(clang_path, argv);
  fprintf(stderr, "header_parser: execv failed: %s\n", strerror(errno));
  return 2;
}
