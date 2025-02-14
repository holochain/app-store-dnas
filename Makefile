.PHONY:			FORCE
SHELL			= bash
NAME			= appstore

# External WASM dependencies
MERE_MEMORY_VERSION	= 0.100.0
MERE_MEMORY_WASM	= zomes/mere_memory.wasm
MERE_MEMORY_API_WASM	= zomes/mere_memory_api.wasm
COOP_CONTENT_VERSION	= 0.8.0
COOP_CONTENT_WASM	= zomes/coop_content.wasm
COOP_CONTENT_CSR_WASM	= zomes/coop_content_csr.wasm

# External DNA dependencies
PORTAL_VERSION		= 0.18.0
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
TARGET_DIR		= zomes/target/wasm32-unknown-unknown/release
COMMON_SOURCE_FILES	= Makefile zomes/Cargo.toml
INT_SOURCE_FILES	= $(COMMON_SOURCE_FILES) \
				dnas/%/types/Cargo.toml dnas/%/types/src/*.rs \
				zomes/%/Cargo.toml zomes/%/src/*.rs \
				zomes/%/src/**
CSR_SOURCE_FILES	= $(COMMON_SOURCE_FILES) $(INT_SOURCE_FILES) \
				zomes/%_csr/Cargo.toml zomes/%_csr/src/*.rs



#
# Project
#
clean:
	rm -rf \
	    node_modules \
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


$(DEVHUB_HAPP):			../devhub-dnas/happ/devhub.happ
	make reset-portal
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
	curl --fail -L "https://github.com/spartan-holochain-counsel/hc-zome-mere-memory/releases/download/v$(MERE_MEMORY_VERSION)/mere_memory.wasm" --output $@
$(MERE_MEMORY_API_WASM):
	curl --fail -L "https://github.com/spartan-holochain-counsel/hc-zome-mere-memory/releases/download/v$(MERE_MEMORY_VERSION)/mere_memory_api.wasm" --output $@

$(COOP_CONTENT_WASM):
	curl --fail -L "https://github.com/holochain/hc-cooperative-content/releases/download/v$(COOP_CONTENT_VERSION)/coop_content.wasm" --output $@
$(COOP_CONTENT_CSR_WASM):
	curl --fail -L "https://github.com/holochain/hc-cooperative-content/releases/download/v$(COOP_CONTENT_VERSION)/coop_content_csr.wasm" --output $@

reset-mere-memory:
	rm -f zomes/mere_memory*.wasm
	make $(MERE_MEMORY_WASM) $(MERE_MEMORY_API_WASM)
reset-coop-content:
	rm -f zomes/coop_content*.wasm
	make $(COOP_CONTENT_WASM) $(COOP_CONTENT_CSR_WASM)
reset-portal:
	rm -f dnas/portal.dna
	make $(PORTAL_DNA)


PRE_EDITION = edition = "2018"
NEW_EDITION = edition = "2021"

PRE_MM_VERSION = mere_memory_types = "0.97"
NEW_MM_VERSION = mere_memory_types = "0.98"

PRE_CRUD_VERSION = hc_crud_caps = "0.17"
NEW_CRUD_VERSION = hc_crud_caps = "0.19"

PRE_HDI_VERSION = hdi = "=0.5.0-dev.12"
NEW_HDI_VERSION = hdi = "=0.5.1"

PRE_HDIE_VERSION = whi_hdi_extensions = "0.12"
NEW_HDIE_VERSION = whi_hdi_extensions = "0.14"

PRE_HDKE_VERSION = whi_hdk_extensions = "0.12"
NEW_HDKE_VERSION = whi_hdk_extensions = "0.14"

PRE_CCSDK_VERSION = hc_coop_content_sdk = "0.7"
NEW_CCSDK_VERSION = hc_coop_content_sdk = "0.8"

GG_REPLACE_LOCATIONS = ':(exclude)*.lock' dnas/*/types zomes/*/

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
   SED_INPLACE := sed -i ''
else
   SED_INPLACE := sed -i
endif

update-all-version: update-mere-memory-version update-crud-version update-hdi-version update-hdk-extensions-version update-hdi-extensions-version update-coop-content-version

update-mere-memory-version:	reset-mere-memory
	rm -f zomes/mere_memory*.wasm
	git grep -l '$(PRE_MM_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs $(SED_INPLACE) 's|$(PRE_MM_VERSION)|$(NEW_MM_VERSION)|g'
update-crud-version:
	git grep -l '$(PRE_CRUD_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs $(SED_INPLACE) 's|$(PRE_CRUD_VERSION)|$(NEW_CRUD_VERSION)|g'
update-hdi-version:
	git grep -l '$(PRE_HDI_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs $(SED_INPLACE) 's|$(PRE_HDI_VERSION)|$(NEW_HDI_VERSION)|g'
update-hdk-extensions-version:
	git grep -l '$(PRE_HDKE_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs $(SED_INPLACE) 's|$(PRE_HDKE_VERSION)|$(NEW_HDKE_VERSION)|g'
update-hdi-extensions-version:
	git grep -l '$(PRE_HDIE_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs $(SED_INPLACE) 's|$(PRE_HDIE_VERSION)|$(NEW_HDIE_VERSION)|g'
update-coop-content-version:	reset-coop-content
	rm -f zomes/coop_content*.wasm
	git grep -l '$(PRE_CCSDK_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs $(SED_INPLACE) 's|$(PRE_CCSDK_VERSION)|$(NEW_CCSDK_VERSION)|g'
update-edition:
	git grep -l '$(PRE_EDITION)' -- $(GG_REPLACE_LOCATIONS) | xargs $(SED_INPLACE) 's/$(PRE_EDITION)/$(NEW_EDITION)/g'

npm-reinstall-local:
	npm uninstall $(NPM_PACKAGE); npm i --save $(LOCAL_PATH)
npm-reinstall-public:
	npm uninstall $(NPM_PACKAGE); npm i --save $(NPM_PACKAGE)

npm-use-app-interface-client-public:
npm-use-app-interface-client-local:
npm-use-app-interface-client-%:
	NPM_PACKAGE=@spartan-hc/app-interface-client LOCAL_PATH=../app-interface-client-js make npm-reinstall-$*

npm-use-backdrop-public:
npm-use-backdrop-local:
npm-use-backdrop-%:
	NPM_PACKAGE=@spartan-hc/holochain-backdrop LOCAL_PATH=../node-backdrop make npm-reinstall-$*

npm-use-apphub-zomelets-public:
npm-use-apphub-zomelets-local:
npm-use-apphub-zomelets-%:
	NPM_PACKAGE=@holochain/apphub-zomelets LOCAL_PATH=../devhub-dnas/dnas/apphub/zomelets make npm-reinstall-$*

npm-use-portal-zomelets-public:
npm-use-portal-zomelets-local:
npm-use-portal-zomelets-%:
	NPM_PACKAGE=@holochain/portal-zomelets LOCAL_PATH=../portal-dna/zomelets make npm-reinstall-$*

npm-use-bundles-public:
npm-use-bundles-local:
npm-use-bundles-%:
	NPM_PACKAGE=@spartan-hc/bundles LOCAL_PATH=../../bundles-js make npm-reinstall-$*



#
# Testing
#
package-lock.json:	package.json
	touch $@
node_modules:		package-lock.json
	npm install
	touch $@
dnas/appstore/%:
	cd dnas/appstore; make $*
test-setup:		node_modules \
			dnas/appstore/zomelets/node_modules

test:
	make -s test-unit
	make -s test-integration
	make -s test-e2e

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
	make -s test-unit-appstore

test-unit-%:
	SRC=dnas/$* make test-crate
test-zome-unit-%:
	cd zomes; cargo test -p $* --quiet

# Integration tests
test-integration:
	make -s test-integration-appstore
	make -s test-integration-viewpoint

DEBUG_LEVEL	       ?= warn
TEST_ENV_VARS		= LOG_LEVEL=$(DEBUG_LEVEL)
MOCHA_OPTS		= -n enable-source-maps -t 10000

test-integration-appstore:	test-setup $(APPSTORE_DNA)
	$(TEST_ENV_VARS) npx mocha $(MOCHA_OPTS) ./tests/integration/test_appstore.js
test-integration-viewpoint:	test-setup $(APPSTORE_DNA)
	$(TEST_ENV_VARS) npx mocha $(MOCHA_OPTS) ./tests/integration/test_controlled_viewpoint.js

# End-2-end tests
test-e2e:
	make -s test-e2e-multi

test-e2e-multi:			test-setup $(APPSTORE_HAPP) $(DEVHUB_HAPP)
	$(TEST_ENV_VARS) npx mocha $(MOCHA_OPTS) ./tests/e2e/test_multiple.js



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


#
# Documentation
#
docs/WEBHAPP_ASSEMBLY.md:
%.md:		node_modules %.template.md
	cd $$(dirname $@); npx mmdc -i $$(basename $*).template.md -o $$(basename $@)

APPSTORE_CSR_DOCS	= target/doc/appstore_csr/index.html

$(APPSTORE_CSR_DOCS):
target/doc/%/index.html:	zomes/%/src/**
	cd zomes; cargo test --doc -p $*
	cd zomes; cargo doc -p $*
	@echo -e "\x1b[37mOpen docs in file://$(shell pwd)/$@\x1b[0m";


docs:				$(APPSTORE_CSR_DOCS)
docs-watch:
	@inotifywait -r -m -e modify		\
		--includei '.*\.rs'		\
		zomes/				\
	| while read -r dir event file; do	\
		echo -e "\x1b[37m$$event $$dir$$file\x1b[0m";\
		make docs;			\
	done
