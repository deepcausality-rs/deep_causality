#ifndef _WIN32
#define _POSIX_C_SOURCE 200809L
#endif

#include <errno.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <io.h>
#include <process.h>
#else
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>
#endif

static int run_process(char *const args[]) {
#ifdef _WIN32
    int status = _spawnv(_P_WAIT, args[0], (const char *const *)args);
    if (status == -1) {
        fprintf(stderr, "link-wrapper: failed to execute %s: %s\n", args[0], strerror(errno));
        return 127;
    }
    return status;
#else
    pid_t pid = fork();
    if (pid == -1) {
        fprintf(stderr, "link-wrapper: failed to fork for %s: %s\n", args[0], strerror(errno));
        return 127;
    }

    if (pid == 0) {
        execv(args[0], args);
        fprintf(stderr, "link-wrapper: failed to execute %s: %s\n", args[0], strerror(errno));
        _exit(127);
    }

    int status = 0;
    while (1) {
        if (waitpid(pid, &status, 0) != -1) {
            break;
        }
        if (errno == EINTR) {
            continue;
        }
        fprintf(stderr, "link-wrapper: failed to wait for %s: %s\n", args[0], strerror(errno));
        return 127;
    }

    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }
    if (WIFSIGNALED(status)) {
        return 128 + WTERMSIG(status);
    }
    return 127;
#endif
}

static int run_process_with_stdout(char *const args[], FILE *output) {
#ifdef _WIN32
    int stdout_fd = _fileno(stdout);
    int saved_stdout = _dup(stdout_fd);
    if (saved_stdout == -1) {
        fprintf(stderr, "link-wrapper: failed to duplicate stdout for %s: %s\n", args[0], strerror(errno));
        return 127;
    }

    fflush(stdout);
    if (_dup2(_fileno(output), stdout_fd) == -1) {
        fprintf(stderr, "link-wrapper: failed to redirect stdout for %s: %s\n", args[0], strerror(errno));
        _close(saved_stdout);
        return 127;
    }

    int status = run_process(args);
    fflush(stdout);
    if (_dup2(saved_stdout, stdout_fd) == -1) {
        fprintf(stderr, "link-wrapper: failed to restore stdout after %s: %s\n", args[0], strerror(errno));
        status = 127;
    }
    _close(saved_stdout);
    return status;
#else
    pid_t pid = fork();
    if (pid == -1) {
        fprintf(stderr, "link-wrapper: failed to fork for %s: %s\n", args[0], strerror(errno));
        return 127;
    }

    if (pid == 0) {
        if (dup2(fileno(output), STDOUT_FILENO) == -1) {
            fprintf(stderr, "link-wrapper: failed to redirect stdout for %s: %s\n", args[0], strerror(errno));
            _exit(127);
        }
        execv(args[0], args);
        fprintf(stderr, "link-wrapper: failed to execute %s: %s\n", args[0], strerror(errno));
        _exit(127);
    }

    int status = 0;
    while (1) {
        if (waitpid(pid, &status, 0) != -1) {
            break;
        }
        if (errno == EINTR) {
            continue;
        }
        fprintf(stderr, "link-wrapper: failed to wait for %s: %s\n", args[0], strerror(errno));
        return 127;
    }

    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }
    if (WIFSIGNALED(status)) {
        return 128 + WTERMSIG(status);
    }
    return 127;
#endif
}

static const char *required_env(const char *name) {
    const char *value = getenv(name);
    if (value == NULL || value[0] == '\0') {
        fprintf(stderr, "link-wrapper: required env var %s is not set\n", name);
        exit(127);
    }
    return value;
}

static int copy_file(const char *source_path, const char *destination_path) {
    FILE *source = fopen(source_path, "rb");
    if (source == NULL) {
        fprintf(stderr, "link-wrapper: failed to open %s: %s\n", source_path, strerror(errno));
        return 1;
    }

    FILE *destination = fopen(destination_path, "wb");
    if (destination == NULL) {
        fprintf(stderr, "link-wrapper: failed to open %s: %s\n", destination_path, strerror(errno));
        fclose(source);
        return 1;
    }

    char buffer[64 * 1024];
    int status = 0;
    while (!feof(source)) {
        size_t bytes_read = fread(buffer, 1, sizeof(buffer), source);
        if (bytes_read > 0 && fwrite(buffer, 1, bytes_read, destination) != bytes_read) {
            fprintf(stderr, "link-wrapper: failed to write %s: %s\n", destination_path, strerror(errno));
            status = 1;
            break;
        }
        if (ferror(source)) {
            fprintf(stderr, "link-wrapper: failed to read %s\n", source_path);
            status = 1;
            break;
        }
    }

    if (fclose(destination) != 0 && status == 0) {
        fprintf(stderr, "link-wrapper: failed to close %s: %s\n", destination_path, strerror(errno));
        status = 1;
    }
    if (fclose(source) != 0 && status == 0) {
        fprintf(stderr, "link-wrapper: failed to close %s: %s\n", source_path, strerror(errno));
        status = 1;
    }
    if (status != 0) {
        remove(destination_path);
    }
    return status;
}

static int has_versioned_symbols(const char *library_path, bool *result) {
    const char *llvm_nm = required_env("LLVM_NM");
    char *nm_args[] = {
        (char *)llvm_nm,
        "--dynamic",
        "--defined-only",
        "--format=just-symbols",
        (char *)library_path,
        NULL,
    };

    FILE *output = tmpfile();
    if (output == NULL) {
        fprintf(stderr, "link-wrapper: failed to create temporary llvm-nm output: %s\n", strerror(errno));
        return 1;
    }

    int status = run_process_with_stdout(nm_args, output);
    if (status != 0) {
        fclose(output);
        return status;
    }

    if (fseek(output, 0, SEEK_SET) != 0) {
        fprintf(stderr, "link-wrapper: failed to rewind llvm-nm output: %s\n", strerror(errno));
        fclose(output);
        return 1;
    }
    *result = false;
    char buffer[4096];
    while (fgets(buffer, sizeof(buffer), output) != NULL) {
        if (strchr(buffer, '@') != NULL) {
            *result = true;
            break;
        }
    }
    if (ferror(output)) {
        fprintf(stderr, "link-wrapper: failed to read llvm-nm output\n");
        status = 1;
    }
    fclose(output);
    return status;
}

static int generate_interface_library(const char *format, const char *input_path, const char *output_path) {
    if (strcmp(format, "elf") == 0) {
        bool has_versions = false;
        int status = has_versioned_symbols(input_path, &has_versions);
        if (status != 0) {
            return status;
        }
        if (has_versions) {
            return copy_file(input_path, output_path);
        }

        const char *llvm_ifs = required_env("LLVM_IFS");
        char *ifs_args[] = {
            (char *)llvm_ifs,
            (char *)input_path,
            "--output-elf",
            (char *)output_path,
            NULL,
        };
        return run_process(ifs_args);
    }

    if (strcmp(format, "tbd") == 0) {
        const char *llvm_readtapi = required_env("LLVM_READTAPI");
        char *readtapi_args[] = {
            (char *)llvm_readtapi,
            "-stubify",
            (char *)input_path,
            "-o",
            (char *)output_path,
            NULL,
        };
        return run_process(readtapi_args);
    }

    fprintf(stderr, "link-wrapper: unsupported interface library format: %s\n", format);
    return 1;
}

int main(int argc, char **argv) {
    (void)argc;

    const char *clangxx = required_env("LLVM_CLANGXX");
    const char *strip_debug_symbols = getenv("LLVM_STRIP_DEBUG_SYMBOLS");
    int should_strip_debug_symbols = strip_debug_symbols != NULL && strip_debug_symbols[0] != '\0';

    argv[0] = (char *)clangxx;
    int status = run_process(argv);
    if (status != 0) {
        return status;
    }

    const char *dsym_path = getenv("LLVM_DSYM_PATH");
    if (dsym_path != NULL && dsym_path[0] != '\0') {
        const char *link_output = required_env("LLVM_LINK_OUTPUT");
        const char *dsymutil = required_env("LLVM_DSYMUTIL");

        char *dsym_args[] = {
            (char *)dsymutil,
            "-o",
            (char *)dsym_path,
            (char *)link_output,
            NULL,
        };

        status = run_process(dsym_args);
        if (status != 0) {
            return status;
        }

        if (should_strip_debug_symbols) {
            const char *strip = required_env("LLVM_STRIP");
            char *strip_args[] = {
                (char *)strip,
                "-S",
                (char *)link_output,
                NULL,
            };

            status = run_process(strip_args);
            if (status != 0) {
                return status;
            }
        }
    }

    const char *generate_interface = getenv("LLVM_GENERATE_INTERFACE_LIBRARY");
    if (generate_interface == NULL || strcmp(generate_interface, "yes") != 0) {
        return 0;
    }

    const char *interface_library_format = required_env("LLVM_INTERFACE_LIBRARY_FORMAT");
    const char *interface_library_input = required_env("LLVM_INTERFACE_LIBRARY_INPUT");
    const char *interface_library_output = required_env("LLVM_INTERFACE_LIBRARY_OUTPUT");
    return generate_interface_library(interface_library_format, interface_library_input, interface_library_output);
}
