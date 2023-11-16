
SHELL			= bash

NAME			= appstore
DEVHUB_HAPP		= tests/devhub.happ
DNAREPO_DNA		= tests/devhub/dnarepo.dna
HAPPS_DNA		= tests/devhub/happs.dna
WEBASSETS_DNA		= tests/devhub/web_assets.dna

APPSTORE_HAPP		= ${NAME}.happ
APPSTORE_DNA		= bundled/appstore.dna

PORTAL_VERSION		= 0.9.2
PORTAL_DNA		= bundled/portal.dna
TARGET			= release

# Zomes (WASM)
APPSTORE_WASM		= zomes/appstore.wasm
APPSTORE_API_WASM	= zomes/appstore_api.wasm

# External Zomes (WASM)
MERE_MEMORY_VERSION	= 0.91.0
MERE_MEMORY_WASM	= zomes/mere_memory.wasm
MERE_MEMORY_API_WASM	= zomes/mere_memory_api.wasm
COOP_CONTENT_WASM	= zomes/coop_content.wasm
COOP_CONTENT_CSR_WASM	= zomes/coop_content_csr.wasm


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
	    $(COOP_CONTENT_WASM) $(COOP_CONTENT_CSR_WASM)

rebuild:			clean build
build:				$(APPSTORE_HAPP)


$(APPSTORE_HAPP):		$(APPSTORE_DNA) $(PORTAL_DNA) bundled/happ.yaml
	hc app pack -o $@ ./bundled/

$(APPSTORE_DNA):		$(APPSTORE_WASM) $(APPSTORE_API_WASM) $(MERE_MEMORY_WASM) $(MERE_MEMORY_API_WASM) $(COOP_CONTENT_WASM) $(COOP_CONTENT_CSR_WASM)
$(PORTAL_DNA):
	wget -O $@ "https://github.com/holochain/portal-dna/releases/download/v$(PORTAL_VERSION)/portal.dna"

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
	curl --fail -L "https://github.com/mjbrisebois/hc-zome-mere-memory/releases/download/v$(MERE_MEMORY_VERSION)/mere_memory.wasm" --output $@
$(MERE_MEMORY_API_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-zome-mere-memory/releases/download/v$(MERE_MEMORY_VERSION)/mere_memory_api.wasm" --output $@

$(COOP_CONTENT_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-cooperative-content/releases/download/v$$(echo $(NEW_CC_VERSION))/coop_content.wasm" --output $@
$(COOP_CONTENT_CSR_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-cooperative-content/releases/download/v$$(echo $(NEW_CC_VERSION))/coop_content_csr.wasm" --output $@

tests/devhub/%.dna:
	wget -O $@ "https://github.com/holochain/devhub-dnas/releases/download/$(DEVHUB_VERSION)/$*.dna"

# $(DEVHUB_HAPP):			$(DNAREPO_DNA) $(HAPPS_DNA) $(WEBASSETS_DNA) $(PORTAL_DNA) tests/devhub/happ.yaml
# 	hc app pack -o $@ ./tests/devhub/
$(DEVHUB_HAPP):			../devhub-dnas/happ/devhub.happ
	cp $< $@

use-local-client:
	cd tests; npm uninstall @whi/holochain-client
	cd tests; npm install --save-dev ../../holochain-client-js/
use-npm-client:
	cd tests; npm uninstall @whi/holochain-client
	cd tests; npm install --save-dev @whi/holochain-client

use-local-backdrop:
	cd tests; npm uninstall @whi/holochain-backdrop
	cd tests; npm install --save-dev ../../node-holochain-backdrop/
use-npm-backdrop:
	cd tests; npm uninstall @whi/holochain-backdrop
	cd tests; npm install --save-dev @whi/holochain-backdrop



#
# Testing
#
test:				test-unit test-integration		test-e2e
test-debug:			test-unit test-integration-debug	test-e2e-debug

test-unit:			test-unit-appstore
test-unit-%:
	cd zomes;		RUST_BACKTRACE=1 cargo test $* -- --nocapture

tests/test.dna:
	cp $(APPSTORE_DNA) $@
tests/test.gz:
	gzip -kc $(APPSTORE_DNA) > $@

# DNAs
test-setup:			tests/node_modules

test-integration:		test-setup test-appstore	test-viewpoint
test-integration-debug:		test-setup test-appstore-debug	test-viewpoint-debug

test-appstore:			test-setup $(APPSTORE_DNA)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_appstore.js
test-appstore-debug:		test-setup $(APPSTORE_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=trace npx mocha integration/test_appstore.js

test-viewpoint:			test-setup $(APPSTORE_DNA)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_controlled_viewpoint.js
test-viewpoint-debug:		test-setup $(APPSTORE_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=trace npx mocha integration/test_controlled_viewpoint.js

test-e2e:			test-setup test-multi
test-e2e-debug:			test-setup test-multi-debug

test-multi:			test-setup $(APPSTORE_HAPP) $(DEVHUB_HAPP)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha e2e/test_multiple.js
test-multi-debug:		test-setup $(APPSTORE_HAPP) $(DEVHUB_HAPP)
	cd tests; RUST_LOG=info LOG_LEVEL=trace npx mocha e2e/test_multiple.js



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

PRE_HDK_VERSION = "=0.2.1"
NEW_HDK_VERSION = "0.2.2"

PRE_HDI_VERSION = "=0.3.1"
NEW_HDI_VERSION = "0.3.2"

PRE_CRUD_VERSION = hc_crud_caps = "0.9.0"
NEW_CRUD_VERSION = hc_crud_caps = "0.10.3"

PRE_MM_VERSION = mere_memory_types = "0.88.0"
NEW_MM_VERSION = mere_memory_types = "0.91.0"

PRE_CC_VERSION = "0.2.1"
NEW_CC_VERSION = "0.2.2"
PRE_CCSDK_VERSION = "0.2.0"
NEW_CCSDK_VERSION = "0.2.1"

PRE_PSDK_VERSION = hc_portal_types = "0.8.0"
NEW_PSDK_VERSION = hc_portal_sdk = "0.1.3"

DEVHUB_VERSION = v0.12.0

GG_REPLACE_LOCATIONS = ':(exclude)*.lock' zomes/*/ *_types/ hc_utils

update-hdk-version:
	git grep -l '$(PRE_HDK_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_HDK_VERSION)|$(NEW_HDK_VERSION)|g'
update-hdi-version:
	git grep -l '$(PRE_HDI_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_HDI_VERSION)|$(NEW_HDI_VERSION)|g'
update-crud-version:
	git grep -l '$(PRE_CRUD_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_CRUD_VERSION)|$(NEW_CRUD_VERSION)|g'
update-mere-memory-version:
	rm -f zomes/mere_memory*.wasm
	git grep -l '$(PRE_MM_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_MM_VERSION)|$(NEW_MM_VERSION)|g'
update-coop-content-version:
	rm -f zomes/coop_content*.wasm
	git grep -l '$(PRE_CCSDK_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_CCSDK_VERSION)|$(NEW_CCSDK_VERSION)|g'
update-portal-version:
	git grep -l '$(PRE_PSDK_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_PSDK_VERSION)|$(NEW_PSDK_VERSION)|g'
	rm -f $(PORTAL_DNA)
