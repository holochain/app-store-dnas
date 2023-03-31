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
const { Holochain,
	Config }			= require('@whi/holochain-backdrop');
const { CruxConfig,
	Translator }			= require('@whi/crux-payload-parser');
const { ConductorError, AdminClient,
	TimeoutError,
	...hc_client }			= require('@whi/holochain-client');

const { expect_reject }			= require('../utils.js');

const delay				= (n) => new Promise(f => setTimeout(f, n));

const DEVHUB_PATH			= path.join( __dirname, "../devhub.happ" );
const APPSTORE_PATH			= path.join( __dirname, "../../appstore.happ" );
const APPSTORE_DNA_PATH			= path.join( __dirname, "../../bundled/appstore.dna" );
const PORTAL_DNA_PATH			= path.join( __dirname, "../../bundled/portal.dna" );

const clients				= {};

let DNAREPO_DNA_HASH;
let HAPPS_DNA_HASH;
let WEBASSETS_DNA_HASH;


async function setup () {
    let hdk_version			= "v0.1.0";
    let zome				= await clients.bobby.devhub.call("dnarepo", "dna_library", "create_zome", {
	"name": "appstore",
	"description": "",
    });
    let zome_version			= await clients.bobby.devhub.call("dnarepo", "dna_library", "create_zome_version", {
	"for_zome": zome.$id,
	"version": "1",
	"ordering": 1,
	"zome_bytes": fs.readFileSync( path.resolve(__dirname, "../../zomes/appstore.wasm") ),
	hdk_version,
    });

    let dna				= await clients.bobby.devhub.call("dnarepo", "dna_library", "create_dna", {
	"name": "appstore",
	"description": "",
    });
    let dna_version			= await clients.bobby.devhub.call("dnarepo", "dna_library", "create_dna_version", {
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

    let gui_file			= await clients.bobby.devhub.call("web_assets", "web_assets", "create_file", {
	"file_bytes": crypto.randomBytes( 1_000 ),
    });
    let gui				= await clients.bobby.devhub.call("happs", "happ_library", "create_gui", {
	"name": "Appstore",
	"description": "",
    });
    let gui_release			= await clients.bobby.devhub.call("happs", "happ_library", "create_gui_release", {
	"version": "1",
	"changelog": "",
	"for_gui": gui.$id,
	"for_happ_releases": [],
	"web_asset_id": gui_file.$addr,
    });

    let happ				= await clients.bobby.devhub.call("happs", "happ_library", "create_happ", {
	"title": "Appstore",
	"subtitle": "",
	"description": "",
    });
    let happ_release			= await clients.bobby.devhub.call("happs", "happ_library", "create_happ_release", {
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

    await clients.bobby.devhub.call("portal", "portal_api", "register_host", {
	"dna": HAPPS_DNA_HASH,
	"granted_functions": {
	    "Listed": [
		[ "happ_library", "get_webhapp_package" ],
		[ "happ_library", "get_happ" ],
		[ "happ_library", "get_happ_release" ],
		[ "happ_library", "get_happ_releases" ],
		[ "happ_library", "get_gui" ],
		[ "happ_library", "get_gui_release" ],
		[ "happ_library", "get_gui_releases" ],
	    ],
	},
    });
    await clients.carol.devhub.call("portal", "portal_api", "register_host", {
	"dna": HAPPS_DNA_HASH,
	"granted_functions": {
	    "Listed": [
		[ "happ_library", "get_webhapp_package" ],
		[ "happ_library", "get_happ" ],
		[ "happ_library", "get_happ_release" ],
		[ "happ_library", "get_happ_releases" ],
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
	"title": "Chess",
	"subtitle": "The classic boardgame",
	"description": "The boardgame known as Chess",
	"icon": new ActionHash( crypto.randomBytes(32) ),
	"publisher": publisher.$id,
	"devhub_address": {
	    "dna": HAPPS_DNA_HASH,
	    "happ": happ.$id,
	    "gui": null,
	},
    });
}

let app;
function download_tests () {
    let available_host;

    async function portal_call ( dna, zome, func, payload, timeout ) {
	if ( available_host === undefined ) {
	    // Get hosts of ...
	    let hosts			= await clients.alice.appstore.call("portal", "portal_api", "get_hosts_for_zome_function", {
		"dna": dna,
		"zome": zome,
		"function": func,
	    });
	    log.info("Found %s hosts for API %s::%s->%s()", hosts.length, dna, zome, func );

	    // Ping hosts ...
	    available_host		= await Promise.any(
		hosts.map(async host => {
		    await clients.alice.appstore.call("portal", "portal_api", "ping", host.author, 1_000 );
		    return new AgentPubKey( host.author );
		})
	    );
	    log.info("Set available host: %s", available_host );
	}

	return await clients.alice.appstore.call("portal", "portal_api", "custom_remote_call", {
	    "host": available_host,
	    "call": {
		"dna": HAPPS_DNA_HASH,
		"zome": zome,
		"function": func,
		"payload": payload,
	    },
	}, timeout );
    }

    it("should find 1 host for happ_library GUI methods", async function () {
	let hosts			= await clients.alice.appstore.call("portal", "portal_api", "get_hosts_for_zome_function", {
	    "dna": HAPPS_DNA_HASH,
	    "zome": "happ_library",
	    "function": "get_gui_releases",
	});

	expect( hosts			).to.have.length( 1 );
    });

    it("should get hApp info", async function () {
	this.timeout( 10_000 );

	const happ			= await portal_call( HAPPS_DNA_HASH, "happ_library", "get_happ", {
	    "id": app.devhub_address.happ,
	}, 10_000 );

	expect( happ.title		).to.be.a("string");
    });

    it("should download DevHub webapp package", async function () {
	this.timeout( 60_000 );

	const happ_releases		= await portal_call( HAPPS_DNA_HASH, "happ_library", "get_happ_releases", {
	    "for_happ": app.devhub_address.happ,
	}, 10_000 );

	expect( happ_releases		).to.have.length( 1 );

	const gui_releases		= await portal_call( HAPPS_DNA_HASH, "happ_library", "get_gui_releases", {
	    "for_gui": happ_releases[0].official_gui,
	}, 10_000 );

	expect( gui_releases		).to.have.length( 1 );

	// Get webhapp package from first host
	const bytes			= await portal_call( HAPPS_DNA_HASH, "happ_library", "get_webhapp_package", {
	    "name": app.title,
	    "happ_release_id": happ_releases[0].$id,
	    "gui_release_id": gui_releases[0].$id,
	}, 30_000 );
	log.info("Received app pacakge with %s bytes", bytes.length );

	expect( bytes.length		).to.be.a("number");
    });

}

let admin;
function errors_tests () {
    it("should fail because 0 hosts registered", async function () {
	const hosts			= await clients.alice.appstore.call("portal", "portal_api", "get_registered_hosts", {
	    "dna": DNAREPO_DNA_HASH,
	});

	expect( hosts			).to.have.length( 0 );
    });

    it("should fail because no host record", async function () {
	this.timeout( 30_000 );

	await expect_reject( async () => {
	    await clients.alice.appstore.call("portal", "portal_api", "custom_remote_call", {
		"host": clients.bobby.devhub.cellAgent(),
		"call": {
		    "dna": DNAREPO_DNA_HASH,
		    "zome": "dna_library",
		    "function": "get_dna",
		    "payload": null,
		},
	    });
	}, "No host record" );
    });

    it("should fail because not unrestricted access", async function () {
	this.timeout( 30_000 );

	await clients.bobby.devhub.call("portal", "portal_api", "register_host", {
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

	await expect_reject( async () => {
	    await clients.alice.appstore.call("portal", "portal_api", "custom_remote_call", {
		"host": clients.bobby.devhub.cellAgent(),
		"call": {
		    "dna": DNAREPO_DNA_HASH,
		    "zome": "dna_library",
		    "function": "get_dna",
		    "payload": null,
		},
	    });
	}, "Access is conditional for DNA" );
    });

    it("should fail because zome/function not granted", async function () {
	this.timeout( 30_000 );

	await expect_reject( async () => {
	    await clients.alice.appstore.call("portal", "portal_api", "custom_remote_call", {
		"host": clients.bobby.devhub.cellAgent(),
		"call": {
		    "dna": HAPPS_DNA_HASH,
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

	let hosts			= await clients.alice.appstore.call("portal", "portal_api", "get_registered_hosts", {
	    "dna": HAPPS_DNA_HASH,
	});
	log.info("Found %s hosts of the 'happs' DNA", hosts.length );

	expect( hosts			).to.have.length( 2 );

	const pings			= hosts.map(host => {
	    return clients.alice.appstore.call("portal", "portal_api", "ping", host.author, 1_000 );
	});

	for ( let p of pings ) {
	    try {
		await p
	    } catch ( err ) {
		if ( !(err instanceof TimeoutError)
		     && !err.message.includes("Disconnected")
		     && !err.message.includes("agent is likely offline") )
		    throw new TypeError(`Expected ping result to be TimeoutError; not type '${err}'`);
	    }
	}
    });
}

const crux				= new CruxConfig();
const interpreter			= new Translator([]);
function organize_clients ( actors ) {
    for ( let name in actors ) {
	if ( clients[ name ] === undefined )
	    clients[ name ]		= {};

	for ( let app_prefix in actors[ name ] ) {
	    log.info("Upgrade client for %s => %s", name, app_prefix );
	    const installation			= actors[ name ][ app_prefix ];
	    const client			= installation.client;
	    clients[ name ][ app_prefix ]	= client;

	    for ( let role_name in installation.dnas ) {
		log.normal("%s new cell : %s (%s)", name, role_name, installation.dnas[ role_name ] );
	    }

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
}

describe("App Store + DevHub", () => {
    const holochain			= new Holochain({
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
	"timeout": 30_000,
    });

    before(async function () {
	this.timeout( 120_000 );

	const actors			= await holochain.backdrop({
	    "devhub":		DEVHUB_PATH,
	}, {
	    "actors": [
		"bobby",
		"carol",
	    ],
	    "network_seed": "test-network",
	});
	organize_clients( actors );

	const install			= await holochain.setupApp(
	    actors.bobby.devhub.app_port,
	    "appstore",
	    "alice",
	    await holochain.admin.generateAgent(),
	    APPSTORE_PATH,
	    {
		"network_seed": "test-network",
	    }
	);
	organize_clients({
	    "alice": {
		"appstore": install,
	    },
	});

	DNAREPO_DNA_HASH		= clients.bobby.devhub._app_schema._dnas.dnarepo._hash;
	HAPPS_DNA_HASH			= clients.bobby.devhub._app_schema._dnas.happs._hash;
	WEBASSETS_DNA_HASH		= clients.bobby.devhub._app_schema._dnas.web_assets._hash;

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.appstore.call("appstore", "appstore_api", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
	{
	    let whoami			= await clients.alice.appstore.call("portal", "portal_api", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
	{
	    let whoami			= await clients.bobby.devhub.call("dnarepo", "dna_library", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
	{
	    let whoami			= await clients.carol.devhub.call("dnarepo", "dna_library", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}

	await setup();

	{
	    const port			= holochain.adminPorts()[0];
	    admin			= new AdminClient( port );
	}

	await admin.disableApp("devhub-carol");
   });

    describe("Download", download_tests.bind( this ) );
    describe("Errors", errors_tests.bind( this ) );

    after(async function () {
	this.timeout( 10_000 );
	await holochain.destroy();
    });

});
