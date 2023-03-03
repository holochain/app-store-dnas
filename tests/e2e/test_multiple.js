const path				= require('path');
const log				= require('@whi/stdlog')(path.basename( __filename ), {
    level: process.env.LOG_LEVEL || 'fatal',
});


const fs				= require('fs');
const crypto				= require('crypto');
const expect				= require('chai').expect;
// const why				= require('why-is-node-running');

const msgpack				= require('@msgpack/msgpack');
const json				= require('@whi/json');
const { ActionHash, EntryHash, AgentPubKey,
	HoloHash }			= require('@whi/holo-hash');
const { Holochain }			= require('@whi/holochain-backdrop');
const { CruxConfig,
	Translator }			= require('@whi/crux-payload-parser');
const { ConductorError, AdminClient,
	TimeoutError,
	...hc_client }			= require('@whi/holochain-client');

const { expect_reject }			= require('../utils.js');

const delay				= (n) => new Promise(f => setTimeout(f, n));

const DEVHUB_PATH			= path.join( __dirname, "../devhub.happ" );
const APPSTORE_DNA_PATH			= path.join( __dirname, "../../bundled/appstore.dna" );
const PORTAL_DNA_PATH			= path.join( __dirname, "../../bundled/portal.dna" );

const clients				= {};

let DNAREPO_DNA_HASH;
let HAPPS_DNA_HASH;
let WEBASSETS_DNA_HASH;


async function setup () {
    let hdk_version			= "v0.1.0";
    let zome				= await clients.alice.devhub.call("dnarepo", "dna_library", "create_zome", {
	"name": "appstore",
	"description": "",
    });
    let zome_version			= await clients.alice.devhub.call("dnarepo", "dna_library", "create_zome_version", {
	"for_zome": zome.$id,
	"version": "1",
	"ordering": 1,
	"zome_bytes": fs.readFileSync( path.resolve(__dirname, "../../zomes/appstore.wasm") ),
	hdk_version,
    });

    let dna				= await clients.alice.devhub.call("dnarepo", "dna_library", "create_dna", {
	"name": "appstore",
	"description": "",
    });
    let dna_version			= await clients.alice.devhub.call("dnarepo", "dna_library", "create_dna_version", {
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
	"origin_time": "2022-02-11T23:05:19.470323Z",
    });

    let gui_file			= await clients.alice.devhub.call("web_assets", "web_assets", "create_file", {
	"file_bytes": crypto.randomBytes( 1_000 ),
    });
    let gui				= await clients.alice.devhub.call("happs", "happ_library", "create_gui", {
	"name": "Appstore",
	"description": "",
    });
    let gui_release			= await clients.alice.devhub.call("happs", "happ_library", "create_gui_release", {
	"version": "1",
	"changelog": "",
	"for_gui": gui.$id,
	"for_happ_releases": [],
	"web_asset_id": gui_file.$addr,
    });

    let happ				= await clients.alice.devhub.call("happs", "happ_library", "create_happ", {
	"title": "Appstore",
	"subtitle": "",
	"description": "",
    });
    let happ_release			= await clients.alice.devhub.call("happs", "happ_library", "create_happ_release", {
	"name": "1",
	"description": "",
	"for_happ": happ.$id,
	"official_gui": gui.$id,
	"ordering": 1,
	"manifest": {
	    "manifest_version": "1",
	    "roles": [
		{
		    "name": "test_dna",
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
		"role_name": "appstore",
		"dna": dna.$id,
		"version": dna_version.$id,
		"wasm_hash": dna_version.wasm_hash,
	    }
	],
    });

    await clients.bobby.appstore.call("portal", "portal_api", "register_host", {
	"dna": HAPPS_DNA_HASH,
	"granted_functions": {
	    "Listed": [
		[ "happ_library", "get_webhapp_package" ],
	    ],
	},
    });
    await clients.carol.appstore.call("portal", "portal_api", "register_host", {
	"dna": HAPPS_DNA_HASH,
	"granted_functions": {
	    "Listed": [
		[ "happ_library", "get_webhapp_package" ],
	    ],
	},
    });

    const publisher			= await clients.alice.appstore.call("appstore", "appstore_api", "create_publisher", {
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

    app					= await clients.alice.appstore.call("appstore", "appstore_api", "create_app", {
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

	// Get hosts of ...
	let hosts			= await clients.alice.appstore.call("appstore", "appstore_api", "get_registered_hosts", "happs" );
	log.info("Found %s hosts of the 'happs' DNA", hosts.length );

	expect( hosts			).to.have.length( 2 );

	// Ping hosts ...
	let available_host		= await Promise.any(
	    hosts.map(async host => {
		await clients.alice.appstore.call("portal", "portal_api", "ping", host.author, 1_000 );
		return new AgentPubKey( host.author );
	    })
	);
	log.info("Received first pong from host %s", available_host );

	// Get webhapp package from first host
	const dna_hash			= await clients.alice.appstore.call("appstore", "appstore_api", "get_dna_hash", "happs" );
	const bytes			= await clients.alice.appstore.call("portal", "portal_api", "custom_remote_call", {
	    "host": available_host,
	    "call": {
		"dna": dna_hash,
		"zome": "happ_library",
		"function": "get_webhapp_package",
		"payload": {
		    "name": app.name,
		    "happ_release_id": app.devhub_address.happ,
		    "gui_release_id": app.devhub_address.gui,
		},
	    },
	}, 30_000 );
	log.info("Received app pacakge with %s bytes", bytes.length );

	expect( bytes.length		).to.be.a("number");
    });

}

let admin;
function errors_tests () {
    it("should fail because of invalid DNA alias ", async function () {
	await expect_reject( async () => {
	    await clients.alice.appstore.call("appstore", "appstore_api", "get_registered_hosts", "invalid" );
	}, "Unknown alias" ); // , "CustomError"
    });

    it("should fail because 0 hosts registered", async function () {
	const hosts			= await clients.alice.appstore.call("appstore", "appstore_api", "get_registered_hosts", "dnarepo" );

	expect( hosts			).to.have.length( 0 );
    });

    it("should fail because no host record", async function () {
	this.timeout( 30_000 );

	const dna_hash			= await clients.alice.appstore.call("appstore", "appstore_api", "get_dna_hash", "dnarepo" );
	await expect_reject( async () => {
	    await clients.alice.appstore.call("portal", "portal_api", "custom_remote_call", {
		"host": clients.bobby.appstore.cellAgent(),
		"call": {
		    "dna": dna_hash,
		    "zome": "dna_library",
		    "function": "get_dna",
		    "payload": null,
		},
	    });
	}, "No host record" );
    });

    it("should fail because not unrestricted access", async function () {
	this.timeout( 30_000 );

	await clients.bobby.appstore.call("portal", "portal_api", "register_host", {
	    "dna": DNAREPO_DNA_HASH,
	    "granted_functions": {
		"Listed": [
		    [ "dna_library", "get_dna" ],
		],
	    },
	    "cap_access": {
		"Transferable": {
		    "secret": new Uint8Array( (new Array(64)).fill(0) ),
		},
	    },
	});

	const dna_hash			= await clients.alice.appstore.call("appstore", "appstore_api", "get_dna_hash", "dnarepo" );
	await expect_reject( async () => {
	    await clients.alice.appstore.call("portal", "portal_api", "custom_remote_call", {
		"host": clients.bobby.appstore.cellAgent(),
		"call": {
		    "dna": dna_hash,
		    "zome": "dna_library",
		    "function": "get_dna",
		    "payload": null,
		},
	    });
	}, "Access is conditional for DNA" );
    });

    it("should fail because zome/function not granted", async function () {
	this.timeout( 30_000 );

	const dna_hash			= await clients.alice.appstore.call("appstore", "appstore_api", "get_dna_hash", "happs" );

	await expect_reject( async () => {
	    await clients.alice.appstore.call("portal", "portal_api", "custom_remote_call", {
		"host": clients.bobby.appstore.cellAgent(),
		"call": {
		    "dna": dna_hash,
		    "zome": "happ_library",
		    "function": "create_app",
		    "payload": null,
		},
	    });
	}, "No capability granted for DNA zome/function" );
    });

    it("should fail because all hosts were unreachable", async function () {
	this.timeout( 60_000 );

	await admin.disableApp("devhub-bobby");
	await admin.disableApp("appstore-bobby");

	let hosts			= await clients.alice.appstore.call("appstore", "appstore_api", "get_registered_hosts", "happs" );
	log.info("Found %s hosts of the 'happs' DNA", hosts.length );

	expect( hosts			).to.have.length( 2 );

	const pings			= hosts.map(host => {
	    return clients.alice.appstore.call("portal", "portal_api", "ping", host.author, 1_000 );
	});

	for ( let p of pings ) {
	    try {
		await p
	    } catch ( err ) {
		if ( !(err instanceof TimeoutError) )
		    throw new TypeError(`Expected ping result to be TimeoutError; not type '${err}'`);
	    }
	}
    });
}

describe("App Store + DevHub", () => {
    const crux				= new CruxConfig();
    const holochain			= new Holochain({
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
	"timeout": 30_000,
    });

    before(async function () {
	this.timeout( 120_000 );

	const actors			= await holochain.backdrop({
	    "devhub":		DEVHUB_PATH,
	    "appstore": {
		"appstore":	APPSTORE_DNA_PATH,
		"portal":	PORTAL_DNA_PATH,
	    },
	}, {
	    "actors": [
		"alice",
		"bobby",
		"carol",
	    ],
	});
	const interpreter		= new Translator([]);

	for ( let name in actors ) {
	    if ( clients[ name ] === undefined )
		clients[ name ]		= {};

	    for ( let app_prefix in actors[ name ] ) {
		log.info("Upgrade client for %s => %s", name, app_prefix );
		const client			= actors[ name ][ app_prefix ].client;
		clients[ name ][ app_prefix ]	= client;

		client.addProcessor("output", (essence, req) => {
		    if ( !( req.dna === "portal"
			    && req.zome === "portal_api"
			    && req.func === "custom_remote_call"
			  ) )
			return essence;

		    let pack;
		    try {
			log.debug("Portal wrapper (%s) with metadata: %s", essence.type, essence.metadata, essence.payload );
			pack			= interpreter.parse( essence );
		    } catch ( err ) {
			log.error("Error unwrapping portal response response:", err );
			return essence;
		    }

		    const payload		= pack.value();

		    if ( payload instanceof Error )
			throw payload;

		    return payload;
		});

		crux.upgrade( client );
	    }
	}

	DNAREPO_DNA_HASH		= clients.alice.devhub._app_schema._dnas.dnarepo._hash;
	HAPPS_DNA_HASH			= clients.alice.devhub._app_schema._dnas.happs._hash;
	WEBASSETS_DNA_HASH		= clients.alice.devhub._app_schema._dnas.web_assets._hash;

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.appstore.call("appstore", "appstore_api", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}

	await setup();

	const port			= holochain.adminPorts()[0];
	admin				= new AdminClient( port );

	await admin.disableApp("devhub-carol");
	await admin.disableApp("appstore-carol");
   });

    describe("Download", download_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async function () {
	this.timeout( 10_000 );
	await holochain.destroy();
    });

});
