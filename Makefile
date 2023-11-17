.PHONY:			FORCE
SHELL			= bash
NAME			= appstore

# External WASM dependencies
MERE_MEMORY_VERSION	= 0.91.0
MERE_MEMORY_WASM	= zomes/mere_memory.wasm
MERE_MEMORY_API_WASM	= zomes/mere_memory_api.wasm
COOP_CONTENT_WASM	= zomes/coop_content.wasm
COOP_CONTENT_CSR_WASM	= zomes/coop_content_csr.wasm

# External DNA dependencies
PORTAL_VERSION		= 0.9.2
PORTAL_DNA		= dnas/portal.dna

# External hApp dependencies
DEVHUB_HAPP		= tests/devhub.happ


# hApp
APPSTORE_HAPP		= happ/${NAME}.happ

# DNAs
APPSTORE_DNA		= dnas/appstore.dna

# Integrity Zomes
APPSTORE_WASM		= zomes/appstore.wasm

# Coordinator WASMs
APPSTORE_CSR_WASM	= zomes/appstore_csr.wasm

TARGET			= release
TARGET_DIR		= target/wasm32-unknown-unknown/release
COMMON_SOURCE_FILES	= Makefile zomes/Cargo.toml
INT_SOURCE_FILES	= $(COMMON_SOURCE_FILES) \
				dnas/%/types/Cargo.toml dnas/%/types/src/*.rs \
				zomes/%/Cargo.toml zomes/%/src/*.rs \
				zomes/%/src/**
CSR_SOURCE_FILES	= $(COMMON_SOURCE_FILES) $(INT_SOURCE_FILES) \
				zomes/%_csr/Cargo.toml zomes/%_csr/src/*.rs \
				dnas/%/sdk/Cargo.toml dnas/%/sdk/src/*.rs



#
# Project
#
clean:
	rm -rf \
	    tests/node_modules \
	    .cargo \
	    target \
	    zomes/target \
	    $(APPSTORE_HAPP) \
	    $(APPSTORE_DNA) $(PORTAL_DNA) \
	    $(APPSTORE_WASM) $(APPSTORE_CSR_WASM) \
	    $(MERE_MEMORY_WASM) $(MERE_MEMORY_API_WASM) \
	    $(COOP_CONTENT_WASM) $(COOP_CONTENT_CSR_WASM)

rebuild:			clean build
build:				$(APPSTORE_HAPP)


$(APPSTORE_HAPP):		$(APPSTORE_DNA) $(PORTAL_DNA) happ/happ.yaml
	hc app pack -o $@ ./happ/

$(APPSTORE_DNA):		$(APPSTORE_WASM) $(APPSTORE_CSR_WASM) $(MERE_MEMORY_WASM) $(MERE_MEMORY_API_WASM) $(COOP_CONTENT_WASM) $(COOP_CONTENT_CSR_WASM)
$(PORTAL_DNA):
	wget -O $@ "https://github.com/holochain/portal-dna/releases/download/v$(PORTAL_VERSION)/portal.dna"


# zomes/%/Cargo.lock:
# 	touch $@

# $(DEVHUB_HAPP):			$(DNAREPO_DNA) $(HAPPS_DNA) $(WEBASSETS_DNA) $(PORTAL_DNA) tests/devhub/happ.yaml
# 	hc app pack -o $@ ./tests/devhub/
$(DEVHUB_HAPP):			../devhub-dnas/happ/devhub.happ
	cp $< $@

dnas/%.dna:			dnas/%/dna.yaml
	@echo "Packaging '$*': $@"
	@hc dna pack -o $@ dnas/$*

zomes/%.wasm:			$(TARGET_DIR)/%.wasm
	@echo -e "\x1b[38;2mCopying WASM ($<) to 'zomes' directory: $@\x1b[0m"; \
	cp $< $@

$(TARGET_DIR)/%.wasm:		$(INT_SOURCE_FILES)
	rm -f zomes/$*.wasm
	@echo -e "\x1b[37mBuilding zome '$*' -> $@\x1b[0m";
	cd zomes; \
	RUST_BACKTRACE=1 cargo build --release \
	    --target wasm32-unknown-unknown \
	    --package $*
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time
$(TARGET_DIR)/%_csr.wasm:	$(CSR_SOURCE_FILES)
	rm -f zomes/$*_csr.wasm
	@echo -e "\x1b[37mBuilding zome '$*_csr' -> $@\x1b[0m";
	cd zomes; \
	RUST_BACKTRACE=1 cargo build --release \
	    --target wasm32-unknown-unknown \
	    --package $*_csr
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time

$(MERE_MEMORY_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-zome-mere-memory/releases/download/v$(MERE_MEMORY_VERSION)/mere_memory.wasm" --output $@
$(MERE_MEMORY_API_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-zome-mere-memory/releases/download/v$(MERE_MEMORY_VERSION)/mere_memory_api.wasm" --output $@

$(COOP_CONTENT_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-cooperative-content/releases/download/v$$(echo $(NEW_CC_VERSION))/coop_content.wasm" --output $@
$(COOP_CONTENT_CSR_WASM):
	curl --fail -L "https://github.com/mjbrisebois/hc-cooperative-content/releases/download/v$$(echo $(NEW_CC_VERSION))/coop_content_csr.wasm" --output $@


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

npm-reinstall-local:
	cd tests; npm uninstall $(NPM_PACKAGE); npm i --save $(LOCAL_PATH)
npm-reinstall-public:
	cd tests; npm uninstall $(NPM_PACKAGE); npm i --save $(NPM_PACKAGE)
npm-use-app-interface-client-public:
npm-use-app-interface-client-local:
npm-use-app-interface-client-%:
	NPM_PACKAGE=@spartan-hc/app-interface-client LOCAL_PATH=../../app-interface-client-js make npm-reinstall-$*



#
# Testing
#
%/package-lock.json:	%/package.json
	touch $@
%/node_modules:		%/package-lock.json
	cd $*; npm install
	touch $@
test-setup:		tests/node_modules \
			dnas/appstore/zomelets/node_modules

tests/test.dna:
	cp $(APPSTORE_DNA) $@
tests/test.gz:
	gzip -kc $(APPSTORE_DNA) > $@

test:
	make test-unit
	make test-integration

# Unit tests
CRATE_DEBUG_LEVELS	= normal info debug trace
test-crate:
	@if [[ "$(CRATE_DEBUG_LEVELS)" == *"$(DEBUG_LEVEL)"* ]]; then \
		cd $(SRC); RUST_BACKTRACE=1 CARGO_TARGET_DIR=../target cargo test -- --nocapture --show-output; \
	else \
		cd $(SRC); CARGO_TARGET_DIR=../target cargo test --quiet --tests; \
	fi
test-unit:
	SRC=zomes make test-crate
	make test-appstore-unit

test-%hub-unit:
	SRC=dnas/$*hub make test-crate
test-zome-unit-%:
	cd zomes; cargo test -p $* --quiet

# test:				test-unit test-integration		test-e2e
# test-debug:			test-unit test-integration-debug	test-e2e-debug

# test-unit:			test-unit-appstore
# test-unit-%:
# 	cd zomes;		RUST_BACKTRACE=1 cargo test $* -- --nocapture

# Integration tests
test-integration:
	make test-appstore
	make test-viewpoint

DEBUG_LEVEL	       ?= warn
TEST_ENV_VARS		= LOG_LEVEL=$(DEBUG_LEVEL)

test-appstore:			test-setup $(APPSTORE_DNA)
	cd tests; $(TEST_ENV_VARS) npx mocha ./integration/test_appstore.js
test-viewpoint:			test-setup $(APPSTORE_DNA)
	cd tests; $(TEST_ENV_VARS) npx mocha ./integration/test_controlled_viewpoint.js

# End-2-end tests
test-e2e:
	make test-multi

test-multi:			test-setup $(APPSTORE_HAPP) $(DEVHUB_HAPP)
	cd tests; $(TEST_ENV_VARS) npx mocha ./e2e/test_multiple.js



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
