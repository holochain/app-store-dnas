import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-multiple", process.env.LOG_LEVEL );

import * as fs				from 'node:fs/promises';
import path				from 'path';
import crypto				from 'crypto';
import { expect }			from 'chai';
import { faker }			from '@faker-js/faker';
// import why				from 'why-is-node-running';

import json				from '@whi/json';
import {
    Bundle,
}					from '@spartan-hc/bundles';
import {
    HoloHash,
    DnaHash, AgentPubKey,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash';

import { Holochain }			from '@spartan-hc/holochain-backdrop';

import {
    AppHubCell,
    DnaHubCell,
    ZomeHubCell,
}					from '@holochain/apphub-zomelets';
import {
    AppStoreCell,
}					from '@holochain/appstore-zomelets';
import {
    PortalCell,
}					from '@holochain/portal-zomelets';
import {
    AppInterfaceClient,
}					from '@spartan-hc/app-interface-client';

import {
    expect_reject,
    linearSuite,
    createAppInput,
    createAppVersionInput,
    createPublisherInput,
    sha256,
}					from '../utils.js';


const __dirname				= path.dirname( new URL(import.meta.url).pathname );
const DEVHUB_PATH			= path.join( __dirname, "../devhub.happ" );
const APPSTORE_PATH			= path.join( __dirname, "../../happ/appstore.happ" );
const APPSTORE_DNA_PATH			= path.join( __dirname, "../../dnas/appstore.dna" );
const PORTAL_DNA_PATH			= path.join( __dirname, "../../dnas/portal.dna" );

const network_seed			= crypto.randomBytes( 8 ).toString("hex");
const holochain				= new Holochain({
    "timeout": 60_000,
    "default_stdout_loggers": log.level_rank > 3,
});

let app_port;
let client;
let alice_client;
let bobby_client;
let carol_client;

let alice_appstore_csr;
let alice_portal_csr;

let bobby_zomehub_csr;
let bobby_dnahub_csr;
let bobby_apphub_csr;
let bobby_portal_csr;

let carol_zomehub_csr;
let carol_dnahub_csr;
let carol_apphub_csr;
let carol_portal_csr;


describe("App Store + DevHub", () => {

    before(async function () {
	this.timeout( 300_000 );

	const appstore_installs		= await holochain.install([
	    "bobby",
	    "carol",
	], [
	    {
		"app_name":	"devhub",
		"bundle":	DEVHUB_PATH,
	    },
	], {
	    network_seed,
	});

	app_port			= await holochain.ensureAppPort();

	client				= new AppInterfaceClient( app_port, {
	    "logging": process.env.LOG_LEVEL || "normal",
	});

	const bobby_token		= appstore_installs.bobby.devhub.auth.token;
	bobby_client			= await client.app( bobby_token );

	const carol_token		= appstore_installs.carol.devhub.auth.token;
	carol_client			= await client.app( carol_token );

	{
	    const {
		zomehub,
		dnahub,
		apphub,
		portal,
	    }				= bobby_client.createInterface({
		"zomehub":	ZomeHubCell,
		"dnahub":	DnaHubCell,
		"apphub":	AppHubCell,
		"portal":	PortalCell,
	    });

	    bobby_zomehub_csr		= zomehub.zomes.zomehub_csr.functions;
	    bobby_dnahub_csr		= dnahub.zomes.dnahub_csr.functions;
	    bobby_apphub_csr		= apphub.zomes.apphub_csr.functions;
	    bobby_portal_csr		= portal.zomes.portal_csr.functions;
	}

	{
	    const {
		zomehub,
		dnahub,
		apphub,
		portal,
	    }				= carol_client.createInterface({
		"zomehub":	ZomeHubCell,
		"dnahub":	DnaHubCell,
		"apphub":	AppHubCell,
		"portal":	PortalCell,
	    });

	    carol_zomehub_csr		= zomehub.zomes.zomehub_csr.functions;
	    carol_dnahub_csr		= dnahub.zomes.dnahub_csr.functions;
	    carol_apphub_csr		= apphub.zomes.apphub_csr.functions;
	    carol_portal_csr		= portal.zomes.portal_csr.functions;
	}

	const devhub_installs		= await holochain.install([
	    "alice",
	], [
	    {
		"app_name": "appstore",
		"bundle": APPSTORE_PATH,
	    },
	], {
	    network_seed,
	});

        log.normal("App Store installations: %s", json.debug(appstore_installs) );
        log.normal("DevHub    installations: %s", json.debug(devhub_installs) );

	const alice_token		= devhub_installs.alice.appstore.auth.token;
	alice_client			= await client.app( alice_token );

	{
	    const {
		appstore,
		portal,
	    }				= alice_client.createInterface({
		"appstore":	AppStoreCell,
		"portal":	PortalCell,
	    })

	    alice_appstore_csr		= appstore.zomes.appstore_csr.functions;
	    alice_portal_csr		= portal.zomes.portal_csr.functions;
	}

	const portal_proxy		= async function ( role, dna, zome, func, args, options ) {
	    // 'this' == CallContext instance
	    this.log.trace("[virtual] cell (%s) call input:", ...arguments );
	    return await alice_portal_csr.remote_call.call( this, {
		"dna": dna,
		"zome": zome,
		"function": func,
		"payload": args,
	    });
	};

	// Create the proxies for virtual cells
	alice_client.createVirtualCells({
	    "apphub": portal_proxy,
	    "dnahub": portal_proxy,
	    "zomehub": portal_proxy,
	});

	// Must call whoami for all of bobby and carol's cells to ensure that inits have registered
	// them as hosts in portal
	await bobby_portal_csr.whoami();
	await bobby_zomehub_csr.whoami();
	await bobby_dnahub_csr.whoami();
	await bobby_apphub_csr.whoami();

	await carol_portal_csr.whoami();
	await carol_zomehub_csr.whoami();
	await carol_dnahub_csr.whoami();
	await carol_apphub_csr.whoami();

	// Call each whoami to ensure inits are complete for more predictable test timing
	await alice_portal_csr.whoami();
	await alice_appstore_csr.whoami();

	await setup();

	await holochain.admin.disableApp("devhub-carol");
    });

    linearSuite("Download", download_tests.bind( this ) );
    linearSuite("Errors", errors_tests.bind( this ) );

    after(async function () {
	this.timeout( 10_000 );
	await holochain.destroy();
    });

});


let src_bundle;
let src_happ_bundle;
let webapp_v1;
let pack_v1;
let version_v1;
let app_v1;
let app_version_v1;

async function setup () {
    src_bundle				= Bundle.createWebhapp({
	"name": "fake-webhapp-1",
	"ui": {
	    "bytes": new Uint8Array( Array( 1_000 ).fill( 1 ) ),
	},
	"happ_manifest": {
	    "bytes": await fs.readFile( APPSTORE_PATH ),
	},
    });

    const bundle_bytes			= src_bundle.toBytes();

    webapp_v1				= await bobby_apphub_csr.save_webapp( bundle_bytes );
    log.normal("WebApp entry: %s", json.debug(webapp_v1) );

    // Setup source bundle for comparison later
    {
	src_happ_bundle			= src_bundle.happ();

	src_happ_bundle.dnas().forEach( (dna_bundle, i) => {
	    const role_manifest		= src_happ_bundle.manifest.roles[i];
	    const rpath			= role_manifest.dna.bundled;

	    // Replace DNA bytes with deterministic bytes
	    src_happ_bundle.resources[ rpath ]	= dna_bundle.toBytes({ sortKeys: true });
	});

	{
	    const rpath			= src_bundle.manifest.happ_manifest.bundled;
	    src_bundle
		.resources[ rpath ]	= src_happ_bundle.toBytes({ sortKeys: true });
	}
    }

    pack_v1				= await bobby_apphub_csr.create_webapp_package({
	"title": faker.commerce.productName(),
	"subtitle": faker.lorem.sentence(),
	"description": faker.lorem.paragraphs( 2 ),
	"icon": crypto.randomBytes( 1_000 ),
	"source_code_uri": faker.internet.url(),
    });
    log.normal("WebApp Package: %s", json.debug(pack_v1) );

    version_v1				= await bobby_apphub_csr.create_webapp_package_version({
	"version": "0.1.0",
	"for_package": pack_v1.$id,
	"webapp": webapp_v1.$addr,
	"source_code_revision_uri": faker.internet.url(),
    });
    log.normal("WebApp Package Version [v0.1.0]: %s", json.debug(version_v1) );

    await bobby_apphub_csr.create_webapp_package_version({
	"version": "0.2.0",
	"for_package": pack_v1.$id,
	"webapp": webapp_v1.$addr,
	"source_code_revision_uri": faker.internet.url(),
    });

    const publisher			= await alice_appstore_csr.create_publisher(
	createPublisherInput()
    );
    log.normal("Publisher: %s", json.debug(publisher) );

    const app_input			= createAppInput({
	"publisher": publisher.$id,
	"apphub_hrl": {
	    "dna": bobby_client.roles.apphub,
	    "target": pack_v1.$id,
	},
	"apphub_hrl_hash": pack_v1.$addr,
    });
    app_v1				= await alice_appstore_csr.create_app( app_input );
    log.normal("App: %s", json.debug(app_v1) );

    const app_version_input		= createAppVersionInput({
	"version": version_v1.version,
	"for_app": app_v1.$id,
	"apphub_hrl": {
	    "dna": bobby_client.roles.apphub,
	    "target": version_v1.$id,
	},
	"apphub_hrl_hash": version_v1.$addr,
    });
    app_version_v1			= await alice_appstore_csr.create_app_version( app_version_input );
    log.normal("App Version: %s", json.debug(app_version_v1) );
}

function download_tests () {

    it("should find 2 host for AppHub", async function () {
	let hosts			= await alice_portal_csr.get_registered_hosts( bobby_client.roles.apphub );

	expect( hosts			).to.have.length( 2 );
    });

    it("should find 2 hosts for AppHub zome/function", async function () {
	let hosts			= await alice_portal_csr.get_hosts_for_zome_function({
	    "dna": bobby_client.roles.apphub,
	    "zome": "apphub_csr",
	    "function": "get_webapp_package_entry",
	});

	expect( hosts			).to.have.length( 2 );
    });

    it("should get webapp package from apphub", async function () {
	const webapp_package_entry	= await alice_appstore_csr.get_apphub_webapp_package({
	    "dna":		app_v1.apphub_hrl.dna,
	    "target":		app_v1.apphub_hrl.target,
	    "hash":		app_v1.apphub_hrl_hash,
	});

	log.normal("WebApp Package entry: %s", json.debug(webapp_package_entry) );
    });

    it("should get webapp package version, webapp and app entry from apphub", async function () {
	const webapp_version_entry	= await alice_appstore_csr.get_apphub_webapp_package_version({
	    "dna":	bobby_client.roles.apphub,
	    "target":	version_v1.$action,
	    "hash":	version_v1.$addr,
	});

	log.normal("WebApp Package Version entry: %s", json.debug(webapp_version_entry) );

	const webapp_entry		= await alice_appstore_csr.get_apphub_webapp({
	    "dna":	bobby_client.roles.apphub,
	    "target":	webapp_version_entry.webapp,
	});

	log.normal("WebApp entry: %s", json.debug(webapp_entry) );

	const app_entry_addr		= webapp_entry.resources[ webapp_entry.manifest.happ_manifest.bundled ];
	const app_entry			= await alice_appstore_csr.get_apphub_app({
	    "dna":	bobby_client.roles.apphub,
	    "target":	app_entry_addr,
	});

	log.normal("App entry: %s", json.debug(app_entry) );
    });

    it("should get UI and memory entry from apphub", async function () {
	const ui_entry_addr		= webapp_v1.resources[ webapp_v1.manifest.ui.bundled ];
	const ui_entry			= await alice_appstore_csr.get_apphub_ui({
	    "dna":	bobby_client.roles.apphub,
	    "target":	ui_entry_addr,
	});

	log.normal("UI entry: %s", json.debug(ui_entry) );

	const memory_entry		= await alice_appstore_csr.get_apphub_memory({
	    "dna":	bobby_client.roles.apphub,
	    "target":	ui_entry.mere_memory_addr,
	});

	log.normal("Mere Memory entry: %s", json.debug(memory_entry) );
    });

    it("should get hApp info", async function () {
	const pack			= await app_v1.$getWebAppPackage();

	log.normal("App's WebApp Package: %s", json.debug(pack) );

	expect( pack.title		).to.be.a("string");
    });

    it("should check for new versions", async function () {
	const versions			= await app_v1.$getWebAppPackageVersions();

	log.normal("App's WebApp Package Versions (sorted): %s", json.debug(versions) );

	expect( versions		).to.have.length( 2 );
    });

    it("should get app versions", async function () {
	const versions			= await app_v1.$getVersions();

	log.normal("App's Versions (sorted): %s", json.debug(versions) );

	expect( versions		).to.have.length( 1 );
    });

    it("should download DevHub webapp package", async function () {
	this.timeout( 120_000 );

	const latest_version		= await app_v1.$getLatestVersion();
	const new_bundle_bytes		= await latest_version.$getBundle();
	const new_bundle		= new Bundle( new_bundle_bytes, "webhapp" );
	log.normal("App's latest webhapp bundle: %s", json.debug(new_bundle) );

	const new_happ_bundle		= new_bundle.happ();
	// const src_happ_bundle		= src_bundle.happ();
	const src_happ_roles		= src_happ_bundle.roles();

	new_happ_bundle.dnas().forEach( (new_dna_bundle, i) => {
	    // Add integrity's 'dependencies' field back in for comparing against source bundle;
	    // which has the field because `hc` bundler adds it.
	    for ( let zome_manifest of new_dna_bundle.manifest.integrity.zomes ) {
		zome_manifest.dependencies	= null;
	    }

	    const role_manifest		= new_happ_bundle.manifest.roles[i];

	    // Compare source DNAs
	    const src_dna_bundle	= src_happ_roles[i].bundle();

	    const src_dna_manifest	= src_dna_bundle.manifest.toJSON();
	    const new_dna_manifest	= new_dna_bundle.manifest.toJSON();

	    log.normal("Comparing role '%s' DNA", role_manifest.name );
	    expect( new_dna_manifest	).to.deep.equal( src_dna_manifest );

	    const src_dna_bytes		= src_dna_bundle.toBytes({ sortKeys: true });
	    const new_dna_bytes		= new_dna_bundle.toBytes({ sortKeys: true });

	    expect(
		sha256( new_dna_bytes )
	    ).to.equal(
		sha256( src_dna_bytes )
	    );

	    const rpath			= role_manifest.dna.bundled;
	    new_happ_bundle
		.resources[ rpath ]	= new_dna_bytes;
	});

	// Compare source happ
	log.normal("Comparing happ");
	expect(
	    src_happ_bundle.toJSON()
	).to.deep.equal(
	    new_happ_bundle.toJSON()
	);

	const src_happ_manifest		= src_happ_bundle.manifest.toJSON();
	const new_happ_manifest		= new_happ_bundle.manifest.toJSON();

	expect( src_happ_manifest	).to.deep.equal( new_happ_manifest );

	const src_happ_bytes		= src_bundle.resources[ src_bundle.manifest.happ_manifest.bundled ];

	const src_happ_msgpack_bytes	= Bundle.gunzip( src_happ_bytes );
	const src_happ_repack_mp_bytes	= src_happ_bundle.toEncoded({ sortKeys: true });
	const new_happ_msgpack_bytes	= new_happ_bundle.toEncoded({ sortKeys: true });

	const src_happ_content		= Bundle.msgpackDecode( src_happ_msgpack_bytes );
	const src_happ_repack_content	= Bundle.msgpackDecode( src_happ_repack_mp_bytes );
	const new_happ_content		= Bundle.msgpackDecode( new_happ_msgpack_bytes );

	expect(
	    src_happ_content
	).to.deep.equal(
	    new_happ_content
	);

	expect(
	    src_happ_content
	).to.deep.equal(
	    src_happ_repack_content
	);

	expect(
	    sha256( src_happ_msgpack_bytes )
	).to.equal(
	    sha256( src_happ_repack_mp_bytes )
	);

	expect(
	    sha256( src_happ_msgpack_bytes )
	).to.equal(
	    sha256( new_happ_msgpack_bytes )
	);

	const new_happ_bytes		= new_happ_bundle.toBytes({ sortKeys: true });

	expect(
	    (new Bundle(src_happ_bytes))
	).to.deep.equal(
	    src_happ_bundle
	);

	// Repacked src happ bundle
	expect(
	    sha256( src_happ_bytes )
	).to.equal(
	    sha256( src_happ_bundle.toBytes({ sortKeys: true }) )
	);

	const src_happ_hash		= sha256( src_happ_bytes );
	const new_happ_hash		= sha256( new_happ_bytes );

	expect( src_happ_hash		).to.equal( new_happ_hash );

	{
	    const rpath			= new_bundle.manifest.happ_manifest.bundled;
	    new_bundle
		.resources[ rpath ]	= new_happ_bytes;
	}

	// Compare source UI
	log.normal("Comparing UI");
	const src_ui			= src_bundle.ui();
	const src_ui_hash		= sha256( src_ui );

	const new_ui			= new_bundle.ui();
	const new_ui_hash		= sha256( new_ui );

	expect( src_ui_hash		).to.equal( new_ui_hash );

	// Compare source webhapp
	log.normal("Comparing webhapp");
	const src_manifest		= src_bundle.manifest.toJSON();
	const new_manifest		= new_bundle.manifest.toJSON();

	expect( src_manifest		).to.deep.equal( new_manifest );

	const src_msgpack_hash		= sha256( src_bundle.toEncoded({ sortKeys: true }) );
	const new_msgpack_hash		= sha256( new_bundle.toEncoded({ sortKeys: true }) );

	expect( src_msgpack_hash	).to.equal( new_msgpack_hash );
    });

    it("should get DevHub webapp asset", async function () {
	this.timeout( 60_000 );

	const latest_version		= await app_v1.$getLatestVersion();
	const asset			= await latest_version.$getAsset();

	log.normal("App's latest webhapp asset: %s", json.debug(asset) );
    });

}

let admin;
function errors_tests () {
    it("should fail because 0 hosts registered", async function () {
	const hosts			= await alice_portal_csr.get_registered_hosts({
	    "dna": alice_client.roles.appstore,
	});

	expect( hosts			).to.have.length( 0 );
    });

    it("should fail because no host record", async function () {
	this.timeout( 30_000 );

	await expect_reject( async () => {
	    await alice_portal_csr.custom_remote_call({
		"host": bobby_client.agent_id,
		"call": {
		    "dna": alice_client.roles.appstore,
		    "zome": "appstore_csr",
		    "function": "get_app",
		    "payload": null,
		},
	    });
	}, "No host record" );
    });

    it("should fail because not unrestricted access", async function () {
	this.timeout( 30_000 );

	await bobby_portal_csr.register_host({
	    "dna": bobby_client.roles.portal,
	    "zomes": {
		"conditional_zome": [
		    "some_func",
		],
	    },
	    "cap_access": {
		"Transferable": {
		    "secret": new Uint8Array( (new Array(64)).fill(0) ),
		},
	    },
	});

	await expect_reject( async () => {
	    await alice_portal_csr.custom_remote_call({
		"host": bobby_client.agent_id,
		"call": {
		    "dna": bobby_client.roles.portal,
		    "zome": "conditional_zome",
		    "function": "some_func",
		    "payload": null,
		},
	    });
	}, "Access is conditional for DNA" );
    });

    it("should fail because zome/function not granted", async function () {
	this.timeout( 30_000 );

	await expect_reject( async () => {
	    await alice_portal_csr.custom_remote_call({
		"host": bobby_client.agent_id,
		"call": {
		    "dna": bobby_client.roles.apphub,
		    "zome": "apphub_csr",
		    "function": "create_webapp",
		    "payload": null,
		},
	    });
	}, "No capability granted for DNA zome/function" );
    });

    it("should fail because requested entry was corrupted", async function () {
	this.timeout( 30_000 );

	const app_entry			= await alice_appstore_csr.get_app( app_v1.$id );
	const target_hash		= app_entry.apphub_hrl_hash;

	const webapp_package		= await alice_appstore_csr.call_apphub_zome_function({
	    "dna":		app_entry.apphub_hrl.dna,
	    "zome":		"apphub_csr",
	    "function":		"get_webapp_package",
	    "args":		app_entry.apphub_hrl.target,
	});
	const good_hash			= await alice_appstore_csr.hash_webapp_package_entry(
	    webapp_package
	);

	expect( target_hash		).to.deep.equal( good_hash );

	webapp_package.title		= "corrupted";

	const bad_hash			= await alice_appstore_csr.hash_webapp_package_entry(
	    webapp_package
	);

	expect( target_hash		).to.deep.not.equal( bad_hash );
    });

    it("should fail because all hosts were unreachable", async function () {
	this.timeout( 60_000 );

	await holochain.admin.disableApp("devhub-bobby");

	await expect_reject( async () => {
	    await alice_appstore_csr.get_apphub_webapp_package({
		"dna":		app_v1.apphub_hrl.dna,
		"target":	app_v1.apphub_hrl.target,
		"hash":		app_v1.apphub_hrl_hash,
	    });
	}, "hosts are unavailable");
    });
}
