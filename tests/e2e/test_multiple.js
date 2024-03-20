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

import HolochainBackdrop		from '@spartan-hc/holochain-backdrop';
const {
    Holochain,
    HolochainClientLib,
}					= HolochainBackdrop;

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
    createPublisherInput,
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
	this.timeout( 120_000 );

	await holochain.backdrop({
	    "devhub":		DEVHUB_PATH,
	}, {
	    "actors": [
		"bobby",
		"carol",
	    ],
	    network_seed,
	});

	app_port			= await holochain.appPorts()[0];

	client				= new AppInterfaceClient( app_port, {
	    "logging": process.env.LOG_LEVEL || "normal",
	});

	bobby_client			= await client.app( "devhub-bobby" );
	carol_client			= await client.app( "devhub-carol" );

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

	const install			= await holochain.setupApp(
	    app_port,
	    "appstore",
	    "alice",
	    await holochain.admin.generateAgent(),
	    APPSTORE_PATH,
	    {
		network_seed,
	    }
	);

	alice_client			= await client.app( "appstore-alice" );

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


let webapp_v1;
let pack_v1;
let version_v1;

async function setup () {
    const bundle			= Bundle.createWebhapp({
	"name": "fake-webhapp-1",
	"ui": {
	    "bytes": new Uint8Array( Array( 1_000 ).fill( 1 ) ),
	},
	"happ_manifest": {
	    "bytes": await fs.readFile( APPSTORE_PATH ),
	},
    });
    const bundle_bytes			= bundle.toBytes();

    webapp_v1				= await bobby_apphub_csr.save_webapp( bundle_bytes );
    log.normal("WebApp entry: %s", json.debug(webapp_v1) );

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
    app					= await alice_appstore_csr.create_app( app_input );
    log.normal("App: %s", json.debug(app) );
}

let app;
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

    it("should get hApp info", async function () {
	const pack			= await app.$getWebAppPackage();

	log.normal("App's WebApp Package: %s", json.debug(pack) );

	expect( pack.title		).to.be.a("string");
    });

    it("should check for new versions", async function () {
	const versions			= await app.$getWebAppPackageVersions();

	log.normal("App's WebApp Package Versions (sorted): %s", json.debug(versions) );

	expect( versions		).to.have.length( 2 );
    });

    it("should download DevHub webapp package", async function () {
	this.timeout( 30_000 );

	const bundle_bytes		= await app.$getLatestBundle();
	const bundle			= new Bundle( bundle_bytes, "webhapp" );

	log.normal("App's latest webhapp bundle: %s", json.debug(bundle) );
    });

    it("should get ui entry", async function () {
	const ui_entry			= await alice_appstore_csr.call_apphub_zome_function({
	    "dna": bobby_client.roles.apphub,
	    "zome": "apphub_csr",
	    "function": "get_ui_entry",
	    "args": webapp_v1.manifest.ui.ui_entry,
	});

	log.normal("UI entry: %s", json.debug(ui_entry) );
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

	const app_entry			= await alice_appstore_csr.get_app( app.$id );
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
	    await alice_appstore_csr.get_webapp_package( app.$id );
	}, "hosts are unavailable");
    });
}
