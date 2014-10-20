all: $(OUT_DIR)/libhttp_parser.a
.PHONY: deps/http-parser/libhttp_parser.a
CFLAGS :=

ifeq ($(DEBUG),true)
	CFLAGS := "-g"
else
	CFLAGS := "-O3"
endif

$(OUT_DIR)/libhttp_parser.a: deps/http-parser/libhttp_parser.a
	cp $< $@

deps/http-parser/libhttp_parser.a:
	make -C deps/http-parser clean package CFLAGS="-fPIC -fexceptions $(CFLAGS)"
