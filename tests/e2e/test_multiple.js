const path				= require('path');
const log				= require('@whi/stdlog')(path.basename( __filename ), {
    level: process.env.LOG_LEVEL || 'fatal',
});


const fs				= require('fs');
const crypto				= require('crypto');
const expect				= require('chai').expect;
const msgpack				= require('@msgpack/msgpack');
const { ActionHash, EntryHash, AgentPubKey,
	HoloHash }			= require('@whi/holo-hash');
const { Holochain }			= require('@whi/holochain-backdrop');
const json				= require('@whi/json');
// const why				= require('why-is-node-running');
const { ConductorError,
	AgentClient, AdminClient,
	...hc_client }			= require('@whi/holochain-client');

const { expect_reject }			= require('../utils.js');
const { backdrop }			= require('../setup.js');

const delay				= (n) => new Promise(f => setTimeout(f, n));

const APPSTORE_DNA_PATH			= path.join( __dirname, "../../bundled/appstore.dna" );
const PORTAL_DNA_PATH			= path.join( __dirname, "../../bundled/portal.dna" );
// DevHUb
const DNAREPO_DNA_PATH			= path.join( __dirname, "../dnarepo.dna" );
const HAPPS_DNA_PATH			= path.join( __dirname, "../happs.dna" );
const WEBASSETS_DNA_PATH		= path.join( __dirname, "../web_assets.dna" );

let DNAREPO_DNA_HASH;
let HAPPS_DNA_HASH;
let WEBASSETS_DNA_HASH;
let clients;

async function setup () {
    let hdk_version		= "v0.0.160";
    let zome			= await clients.alice.call( "dnarepo", "dna_library", "create_zome", {
	"name": "appstore",
	"description": "",
    });
    let zome_version		= await clients.alice.call( "dnarepo", "dna_library", "create_zome_version", {
	"for_zome": zome.$id,
	"version": "1",
	"ordering": 1,
	"zome_bytes": fs.readFileSync( path.resolve(__dirname, "../../zomes/appstore.wasm") ),
	hdk_version,
    });

    let dna			= await clients.alice.call( "dnarepo", "dna_library", "create_dna", {
	"name": "appstore",
	"description": "",
    });
    let dna_version		= await clients.alice.call( "dnarepo", "dna_library", "create_dna_version", {
	"for_dna": dna.$id,
	"version": "1",
	"ordering": 1,
	hdk_version,
	"integrity_zomes": [{
	    "name": "appstore",
	    "zome": new EntryHash( zome_version.for_zome ),
	    "version": zome_version.$id,
	    "resource": new EntryHash( zome_version.mere_memory_addr ),
	    "resource_hash": zome_version.mere_memory_hash,
	}],
	"zomes": [],
    });

    let gui_file		= await clients.alice.call( "web_assets", "web_assets", "create_file", {
	"file_bytes": crypto.randomBytes( 1_000 ),
    });
    let gui			= await clients.alice.call( "happs", "happ_library", "create_gui", {
	"name": "Appstore",
	"description": "",
    });
    let gui_release		= await clients.alice.call( "happs", "happ_library", "create_gui_release", {
	"version": "1",
	"changelog": "",
	"for_gui": gui.$id,
	"for_happ_releases": [],
	"web_asset_id": gui_file.$addr,
    });

    let happ			= await clients.alice.call( "happs", "happ_library", "create_happ", {
	"title": "Appstore",
	"subtitle": "",
	"description": "",
    });
    let happ_release		= await clients.alice.call( "happs", "happ_library", "create_happ_release", {
	"name": "1",
	"description": "",
	"for_happ": happ.$id,
	"official_gui": gui.$id,
	"ordering": 1,
	"manifest": {
	    "manifest_version": "1",
	    "roles": [
		{
		    "id": "test_dna",
		    "dna": {
			"path": "./this/does/not/matter.dna",
		    },
		    "clone_limit": 0,
		},
	    ],
	},
	hdk_version,
	"dnas": [
	    {
		"role_id": "appstore",
		"dna": dna.$id,
		"version": dna_version.$id,
		"wasm_hash": dna_version.wasm_hash,
	    }
	],
    });

    const host			= await clients.bobby.call("portal", "portal_api", "register_host", {
	"dna": HAPPS_DNA_HASH,
	"zome": "happ_library",
	"function": "get_webhapp_package",
    });

    const publisher		= await clients.alice.call("appstore", "appstore_api", "create_publisher", {
	"name": "Holochain",
	"location": {
	    "country": "Gibraltar",
	    "region": "Gibraltar",
	    "city": "Gibraltar",
	},
	"website": {
	    "url": "https://github.com/holochain",
	    "context": "github",
	},
	"icon": new ActionHash( crypto.randomBytes(32) ),
    });

    app				= await clients.alice.call("appstore", "appstore_api", "create_app", {
	"name": "Chess",
	"description": "The boardgame known as Chess",
	"icon": new ActionHash( crypto.randomBytes(32) ),
	"publisher": publisher.$id,
	"devhub_address": {
	    "dna": HAPPS_DNA_HASH,
	    "happ": happ_release.$id,
	    "gui": gui_release.$id,
	},
    });
}

let app;
function download_tests () {

    it("should download DevHub webapp package", async function () {
	this.timeout( 60_000 );

	let bytes		= await clients.alice.call( "appstore", "appstore_api", "get_app_package", {
	    "id": app.$id,
	});

	console.log( bytes );
    });

}

let admin;
function errors_tests () {
    it("should fail because all hosts were unreachable", async function () {
	await admin.disableApp("test-bobby");

	await expect_reject( async () => {
	    await clients.alice.call( "appstore", "appstore_api", "get_app_package", {
		"id": app.$id,
	    });
	}, "WasmError", "All hosts were unreachable" );
    });
}

describe("App Store + DevHub", () => {

    const holochain			= new Holochain({
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
    });

    before(async function () {
	this.timeout( 120_000 );

	clients				= await backdrop( holochain, {
	    "appstore": APPSTORE_DNA_PATH,
	    "portal": PORTAL_DNA_PATH,
	    "dnarepo": DNAREPO_DNA_PATH,
	    "happs": HAPPS_DNA_PATH,
	    "web_assets": WEBASSETS_DNA_PATH,
	}, [
	    "alice",
	    "bobby",
	    "carol",
	]);

	DNAREPO_DNA_HASH		= clients.alice._app_schema._dnas.dnarepo._hash;
	HAPPS_DNA_HASH			= clients.alice._app_schema._dnas.happs._hash;
	WEBASSETS_DNA_HASH		= clients.alice._app_schema._dnas.web_assets._hash;

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.call( "appstore", "appstore_api", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}

	await setup();

	const port			= holochain.adminPorts()[0];
	admin				= new AdminClient( port );

	await admin.disableApp("test-carol");
   });

    describe("Download", download_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});
