all: $(OUT_DIR)/libhttp_parser.a

$(OUT_DIR)/libhttp_parser.a: deps/http-parser/libhttp_parser.a
	cp $< $@

deps/http-parser/libhttp_parser.a:
	make -C deps/http-parser clean package CFLAGS="-fPIC -fexceptions"
