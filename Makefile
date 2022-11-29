
SHELL			= bash

NAME			= appstore
APPSTORE_HAPP		= ${NAME}.happ
APPSTORE_DNA		= bundled/appstore.dna
TARGET			= release

# Zomes (WASM)
APPSTORE_ZOME		= zomes/appstore.wasm
APPSTORE_API_ZOME	= zomes/appstore_api.wasm
PORTAL_ZOME		= zomes/portal.wasm
PORTAL_API_ZOME		= zomes/portal_api.wasm

# External Zomes (WASM)
MERE_MEMORY_WASM	= zomes/mere_memory.wasm
MERE_MEMORY_CORE_WASM	= zomes/mere_memory_core.wasm


#
# Project
#
tests/package-lock.json:	tests/package.json
	touch $@
tests/node_modules:		tests/package-lock.json
	cd tests; \
	npm install
	touch $@
clean:
	rm -rf \
	    tests/node_modules \
	    .cargo \
	    target \
	    zomes/target \
	    $(APPSTORE_HAPP) \
	    $(APPSTORE_DNA) \
	    $(APPSTORE_ZOME) $(APPSTORE_API_ZOME) \
	    $(PORTAL_ZOME) $(PORTAL_API_ZOME)

rebuild:			clean build
build:				$(APPSTORE_HAPP) $(PORTAL_HAPP)


$(APPSTORE_HAPP):		$(APPSTORE_DNA) bundled/happ.yaml
	hc app pack -o $@ ./bundled/

$(APPSTORE_DNA):		$(APPSTORE_ZOME) $(APPSTORE_API_ZOME) $(PORTAL_ZOME) $(PORTAL_API_ZOME)

bundled/%.dna:			bundled/%/dna.yaml
	@echo "Packaging '$*': $@"
	@hc dna pack -o $@ bundled/$*
zomes/%.wasm:			zomes/target/wasm32-unknown-unknown/release/%.wasm
	cp $< $@
zomes/target/wasm32-unknown-unknown/release/%.wasm:	Makefile zomes/%/src/*.rs zomes/%/Cargo.toml zomes/%/Cargo.lock
	@echo "Building  '$*' WASM: $@"; \
	cd zomes; \
	RUST_BACKTRACE=1 CARGO_TARGET_DIR=target cargo build --release \
	    --target wasm32-unknown-unknown \
	    --package $*
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time
zomes/%/Cargo.lock:
	touch $@

# $(MERE_MEMORY_WASM):
# 	curl --fail -L 'https://github.com/mjbrisebois/hc-zome-mere-memory/releases/download/v0.60.1/mere_memory.wasm' --output $@
# $(MERE_MEMORY_CORE_WASM):
# 	curl --fail -L 'https://github.com/mjbrisebois/hc-zome-mere-memory/releases/download/v0.60.1/mere_memory_core.wasm' --output $@



#
# Testing
#
test:				test-unit test-dnas
test-debug:			test-unit test-dnas-debug

test-unit:			test-unit test-unit-appstore
test-unit-%:
	cd zomes;		RUST_BACKTRACE=1 cargo test $* -- --nocapture

tests/test.dna:
	cp $(APPSTORE_DNA) $@
tests/test.gz:
	gzip -kc $(APPSTORE_DNA) > $@

# DNAs
test-setup:			tests/node_modules

test-dnas:			test-setup test-appstore
test-dnas-debug:		test-setup test-appstore-debug


test-appstore:			test-setup $(APPSTORE_DNA)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_appstore.js
test-appstore-debug:		test-setup $(APPSTORE_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=silly npx mocha integration/test_appstore.js



#
# Repository
#
clean-remove-chaff:
	@find . -name '*~' -exec rm {} \;
clean-files:		clean-remove-chaff
	git clean -nd
clean-files-force:	clean-remove-chaff
	git clean -fd
clean-files-all:	clean-remove-chaff
	git clean -ndx
clean-files-all-force:	clean-remove-chaff
	git clean -fdx

PRE_HDK_VERSION = "0.0.151"
NEW_HDK_VERSION = "0.0.160"

PRE_HDI_VERSION = "0.1.1"
NEW_HDI_VERSION = "0.1.8"

PRE_CRUD_VERSION = "0.59.0"
NEW_CRUD_VERSION = "0.68.0"

PRE_MM_VERSION = "0.51.0"
NEW_MM_VERSION = "0.60.0"

GG_REPLACE_LOCATIONS = ':(exclude)*.lock' zomes/*/

update-hdk-version:
	git grep -l $(PRE_HDK_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_HDK_VERSION)/$(NEW_HDK_VERSION)/g'
update-hdi-version:
	git grep -l $(PRE_HDI_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_HDI_VERSION)/$(NEW_HDI_VERSION)/g'
update-crud-version:
	git grep -l $(PRE_CRUD_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_CRUD_VERSION)/$(NEW_CRUD_VERSION)/g'
# update-mere-memory-version:
# 	git grep -l $(PRE_MM_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_MM_VERSION)/$(NEW_MM_VERSION)/g'
