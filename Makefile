
SHELL			= bash

NAME			= appstore
DEVHUB_HAPP		= tests/devhub.happ
APPSTORE_HAPP		= ${NAME}.happ
APPSTORE_DNA		= bundled/appstore.dna
PORTAL_DNA		= bundled/portal.dna
TARGET			= release

# Zomes (WASM)
APPSTORE_WASM		= zomes/appstore.wasm
APPSTORE_API_WASM	= zomes/appstore_api.wasm
PORTAL_WASM		= zomes/portal.wasm
PORTAL_API_WASM		= zomes/portal_api.wasm

# External Zomes (WASM)
MERE_MEMORY_WASM	= zomes/mere_memory.wasm
MERE_MEMORY_API_WASM	= zomes/mere_memory_api.wasm


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
	    $(APPSTORE_DNA) $(PORTAL_DNA) \
	    $(APPSTORE_WASM) $(APPSTORE_API_WASM) \
	    $(MERE_MEMORY_WASM) $(MERE_MEMORY_API_WASM) \
	    $(PORTAL_WASM) $(PORTAL_API_WASM)

rebuild:			clean build
build:				$(APPSTORE_HAPP) $(PORTAL_HAPP)


$(APPSTORE_HAPP):		$(APPSTORE_DNA) $(PORTAL_DNA) bundled/happ.yaml
	hc app pack -o $@ ./bundled/

$(APPSTORE_DNA):		$(APPSTORE_WASM) $(APPSTORE_API_WASM) $(MERE_MEMORY_WASM) $(MERE_MEMORY_API_WASM)
$(PORTAL_DNA):			$(PORTAL_WASM) $(PORTAL_API_WASM)

bundled/%.dna:			bundled/%/dna.yaml
	@echo "Packaging '$*': $@"
	@hc dna pack -o $@ bundled/$*
zomes/%.wasm:			zomes/target/wasm32-unknown-unknown/release/%.wasm
	cp $< $@
zomes/target/wasm32-unknown-unknown/release/%.wasm:	Makefile zomes/%/src/*.rs zomes/%/Cargo.toml zomes/%/Cargo.lock *_types/* *_types/*/*
	@echo "Building  '$*' WASM: $@"; \
	cd zomes; \
	RUST_BACKTRACE=1 CARGO_TARGET_DIR=target cargo build --release \
	    --target wasm32-unknown-unknown \
	    --package $*
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time
zomes/%/Cargo.lock:
	touch $@

$(MERE_MEMORY_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-zome-mere-memory/releases/download/v$$(echo $(NEW_MM_VERSION))/mere_memory_core.wasm" --output $@
$(MERE_MEMORY_API_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-zome-mere-memory/releases/download/v$$(echo $(NEW_MM_VERSION))/mere_memory.wasm" --output $@

$(DEVHUB_HAPP):
	$(error Download missing hApp into location ./$@)
copy-devhub-from-local:
	cp ../devhub-dnas/DevHub.happ $(DEVHUB_HAPP)

use-local-client:
	cd tests; npm uninstall @whi/holochain-client
	cd tests; npm install --save ../../js-holochain-client/whi-holochain-client-0.78.0.tgz
use-npm-client:
	cd tests; npm uninstall @whi/holochain-client
	cd tests; npm install --save @whi/holochain-client

use-local-backdrop:
	cd tests; npm uninstall @whi/holochain-backdrop
	cd tests; npm install --save ../../node-holochain-backdrop/
use-npm-backdrop:
	cd tests; npm uninstall @whi/holochain-backdrop
	cd tests; npm install --save @whi/holochain-backdrop



#
# Testing
#
test:				test-unit test-integration		test-e2e
test-debug:			test-unit test-integration-debug	test-e2e-debug

test-unit:			test-unit test-unit-appstore
test-unit-%:
	cd zomes;		RUST_BACKTRACE=1 cargo test $* -- --nocapture

tests/test.dna:
	cp $(APPSTORE_DNA) $@
tests/test.gz:
	gzip -kc $(APPSTORE_DNA) > $@

# DNAs
test-setup:			tests/node_modules

test-integration:		test-setup test-appstore	test-portal
test-integration-debug:		test-setup test-appstore-debug	test-portal-debug

test-appstore:			test-setup $(APPSTORE_DNA)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_appstore.js
test-appstore-debug:		test-setup $(APPSTORE_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=silly npx mocha integration/test_appstore.js
test-portal:			test-setup $(PORTAL_DNA)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_portal.js
test-portal-debug:		test-setup $(PORTAL_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=silly npx mocha integration/test_portal.js

test-e2e:			test-setup test-multi
test-e2e-debug:			test-setup test-multi-debug

test-multi:			test-setup $(APPSTORE_HAPP)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha e2e/test_multiple.js
test-multi-debug:		test-setup $(APPSTORE_HAPP)
	cd tests; RUST_LOG=info LOG_LEVEL=silly npx mocha e2e/test_multiple.js



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

PRE_HDK_VERSION = "0.1.0-beta-rc.2"
NEW_HDK_VERSION = "0.1.0"

PRE_HDI_VERSION = "0.2.0-beta-rc.2"
NEW_HDI_VERSION = "0.2.0"

PRE_CRUD_VERSION = "0.2.0"
NEW_CRUD_VERSION = "0.3.0"

PRE_MM_VERSION = "0.77.0"
NEW_MM_VERSION = "0.79.0"

GG_REPLACE_LOCATIONS = ':(exclude)*.lock' zomes/*/ *_types/ hc_utils

update-hdk-version:
	git grep -l $(PRE_HDK_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_HDK_VERSION)/$(NEW_HDK_VERSION)/g'
update-hdi-version:
	git grep -l $(PRE_HDI_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_HDI_VERSION)/$(NEW_HDI_VERSION)/g'
update-crud-version:
	git grep -l $(PRE_CRUD_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_CRUD_VERSION)/$(NEW_CRUD_VERSION)/g'
update-mere-memory-version:
	rm -f zomes/mere_memory*.wasm
#	git grep -l $(PRE_MM_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_MM_VERSION)/$(NEW_MM_VERSION)/g'
