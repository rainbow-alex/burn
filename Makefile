LIB_SRC = $(shell find src/libburn/. -type f -name '*.rs')
LIB_RLIB = libburn-2d7926e1-0.1.rlib

## Recipes ##

.PHONY: all clean burn lib tests unit_tests system_tests benchmarks docs reference api_docs

all: burn docs
clean:
	rm -rf build/

lib: build/lib/$(LIB_RLIB)
burn: build/bin/burn

tests: unit_tests system_tests
unit_tests: build/tests/unit_tests
	RUST_TEST_TASKS=1 build/tests/unit_tests
system_tests: burn
	python src/system_tests/run_tests.py
benchmarks: build/tests/benchmarks

docs: reference api_docs
reference: build/doc/reference.html
api_docs: build/doc/api/burn/index.html

## Library ##

build/lib/$(LIB_RLIB): $(LIB_SRC)
	mkdir -p build/lib
	rustc --out-dir build/lib/ src/libburn/lib.rs

## Binaries ##

build/bin/burn: build/lib/$(LIB_RLIB) src/bin/burn.rs
	mkdir -p build/bin
	rustc -L build/lib/ --out-dir build/bin src/bin/burn.rs

## Tests ##

build/tests/unit_tests: $(LIB_SRC)
	mkdir -p build/tests
	rustc --test -o build/tests/unit_tests src/libburn/lib.rs

build/tests/benchmarks: $(LIB_SRC)
	mkdir -p build/tests
	$(RUST_BIN)/rustc --test -o build/tests/benchmarks src/libburn/lib.rs
	RUST_TEST_TASKS=1 build/tests/benchmarks --bench

## Docs ##

build/doc/reference.html: src/doc/reference.md src/doc/burn.sass
	mkdir -p build/doc
	sass --no-cache src/doc/burn.sass build/doc/burn.css
	pandoc --from=markdown --to=html5 --standalone --toc --number-sections --css burn.css -o build/doc/reference.html src/doc/reference.md

build/doc/api/burn/index.html: $(LIB_SRC)
	mkdir -p build/doc/api
	rustdoc -o build/doc/api/ src/libburn/lib.rs

## Misc ##

.PHONY: todo
todo:
	grep -HrnIi --color=always -C1 "todo!" src | sed "s/^/    /"
	grep -HrnIi --color=always "refactor!" src | sed "s/^/    /"
	grep -HrnIi --color=always "optimize!" src | sed "s/^/    /"
	grep -HrnIi --color=always "not_implemented!" src | sed "s/^/    /"
	grep -HrnIi --color=always ".\{101,\}" src/libburn/ | sed "s/^/    /"
