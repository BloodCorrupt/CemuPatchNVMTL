#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>

static char g_target_path[1024] = {0};

static void setup_target(void) {
    if (g_target_path[0] != '\0') return;

    // 1. Check explicit override
    const char *env = getenv("NVMTL_TRANSLATE_OVERRIDE");
    if (env && env[0] != '\0' && access(env, R_OK) == 0) {
        strncpy(g_target_path, env, sizeof(g_target_path) - 1);
        return;
    }

    // 2. Check next to this shim library
    Dl_info info;
    if (dladdr((const void *)&setup_target, &info) && info.dli_fname) {
        char dir[1024];
        strncpy(dir, info.dli_fname, sizeof(dir) - 1);
        char *last_slash = strrchr(dir, '/');
        if (last_slash) {
            *last_slash = '\0';
            snprintf(g_target_path, sizeof(g_target_path), "%s/libnvmtl_translate.dylib", dir);
            if (access(g_target_path, R_OK) == 0) return;
        }
    }

    // 3. Fallback: workspace dist directory
    snprintf(g_target_path, sizeof(g_target_path), "/Users/bloodtek/CemuPatchNVMTL/dist/libnvmtl_translate.dylib");
}

__attribute__((constructor))
static void init(void) {
    setup_target();
    const char *dbg = getenv("CEMU_NVMTL_DEBUG");
    if (dbg && dbg[0] == '1') {
        fprintf(stderr, "[CemuPatchNVMTL] Shim active (PID %d) -> target translator: %s\n", getpid(), g_target_path);
    }
}

void *cemunvmtl_dlopen(const char *path, int mode) {
    setup_target();
    if (path && g_target_path[0] != '\0' && strcmp(path, g_target_path) != 0) {
        if (strstr(path, "libnvmtl_translate.dylib") != NULL) {
            const char *dbg = getenv("CEMU_NVMTL_DEBUG");
            if (dbg && dbg[0] == '1') {
                fprintf(stderr, "[CemuPatchNVMTL] Intercepted dlopen(%s) -> redirected to %s\n", path, g_target_path);
            }
            return dlopen(g_target_path, mode);
        }
    }
    return dlopen(path, mode);
}

int cemunvmtl_access(const char *path, int mode) {
    setup_target();
    if (path && g_target_path[0] != '\0' && strcmp(path, g_target_path) != 0) {
        if (strstr(path, "libnvmtl_translate.dylib") != NULL) {
            return access(g_target_path, mode);
        }
    }
    return access(path, mode);
}

__attribute__((used, section("__DATA,__interpose")))
static struct {
    void *new_func;
    void *orig_func;
} interposers[] = {
    { (void *)cemunvmtl_dlopen, (void *)dlopen },
    { (void *)cemunvmtl_access, (void *)access },
};

