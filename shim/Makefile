CC ?= clang
CFLAGS ?= -O2 -Wall -Wextra
OUT_DIR ?= ../dist

all: $(OUT_DIR)/libcemunvmtl_shim.dylib

$(OUT_DIR)/libcemunvmtl_shim.dylib: cemunvmtl_shim.c
	mkdir -p $(OUT_DIR)
	$(CC) $(CFLAGS) -dynamiclib -o $@ $<
	codesign -f -s - $@

clean:
	rm -f $(OUT_DIR)/libcemunvmtl_shim.dylib

.PHONY: all clean

