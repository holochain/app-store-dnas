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
const { ActionHash, AgentPubKey,
	HoloHash }			= require('@whi/holo-hash');
const { CruxConfig }			= require('@whi/crux-payload-parser');
const { Holochain,
	HolochainClientLib }		= require('@whi/holochain-backdrop');
const { ConductorError,
	...hc_client }			= HolochainClientLib;

const { expect_reject }			= require('../utils.js');

const delay				= (n) => new Promise(f => setTimeout(f, n));

const APPSTORE_DNA_PATH			= path.join( __dirname, "../../bundled/appstore.dna" );
const TEST_DNA_HASH			= "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5";

const clients				= {};


function createPublisherInput ( overrides ) {
    return Object.assign({
	"name": "Holo",
	"location": {
	    "country": "Gibraltar",
	    "region": "Gibraltar",
	    "city": "Gibraltar",
	},
	"website": {
	    "url": "https://github.com/holo-host",
	    "context": "github",
	},
	"icon": crypto.randomBytes(1_000),
	"email": "techservices@holo.host",
	"editors": [
	    new AgentPubKey( crypto.randomBytes(32) )
	],
    }, overrides );
};


let publisher_1;

function publisher_tests () {

    it("should create publisher profile", async function () {
	this.timeout( 10_000 );

	const publisher = publisher_1	= await clients.alice.call("appstore", "appstore_api", "create_publisher", createPublisherInput() );

	// log.debug( json.debug( publisher ) );

	expect( publisher.editors	).to.have.length( 2 );
    });

}


function createAppInput ( overrides ) {
    return Object.assign({
	"title": "Chess",
	"subtitle": "The classic boardgame",
	"description": "The boardgame known as Chess",
	"icon": crypto.randomBytes(1_000),
	"publisher": publisher_1.$id,
	"devhub_address": {
	    "dna": TEST_DNA_HASH,
	    "happ": publisher_1.$id,
	    "gui": publisher_1.$id,
	},
	"editors": [
	    new AgentPubKey( crypto.randomBytes(32) )
	],
    }, overrides );
};


let app_1;

function app_tests () {

    it("should create app profile", async function () {
	this.timeout( 10_000 );

	const app = app_1		= await clients.alice.call("appstore", "appstore_api", "create_app", createAppInput() );

	log.normal("App ID: %s", json.debug( app.$id ) );

	expect( app.editors		).to.have.length( 2 );
    });

}


let group_1;

function createGroupInput ( admins, ...members ) {
    return {
	"admins": admins,
	"members": [ ...members ],

	"published_at":		Date.now(),
	"last_updated":		Date.now(),
	"metadata":		{},
    };
};

function group_tests () {

    it("should create group viewpoint", async function () {
	this.timeout( 10_000 );

	const group_input		= createGroupInput(
	    [
		clients.alice.cellAgent(),
	    ],
	);
	group_1				= await clients.alice.call("appstore", "appstore_api", "create_group", group_input );

	console.log( group_1 );
	log.debug("Group: %s", json.debug( group_1 ) );

	// expect( publisher.editors	).to.have.length( 2 );
    });

    it("should create group viewpoint", async function () {
	{
	    const apps			= await clients.alice.call("appstore", "appstore_api", "get_all_apps" );

	    expect( apps		).to.have.length( 1 );
	}
	const apps			= await clients.alice.call("appstore", "appstore_api", "viewpoint_get_all_apps", group_1.$id );

	log.debug( json.debug( apps ) );

	expect( apps			).to.have.length( 1 );

	const ma_state			= await clients.alice.call("appstore", "appstore_api", "get_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state		).to.be.null;
    });

    it("should remove app from group view", async function () {
	const moderator_action		= await clients.alice.call("appstore", "appstore_api", "update_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	    "message": "App fails to install and developer cannot be contacted",
	    "metadata": {
		"remove": true,
	    },
	});

	log.debug("Removed app: %s", json.debug(moderator_action) );

	{
	    const apps			= await clients.alice.call("appstore", "appstore_api", "viewpoint_get_all_apps", group_1.$id );
	    log.debug( json.debug( apps ) );
	    expect( apps		).to.have.length( 0 );
	}
	{
	    const apps			= await clients.alice.call("appstore", "appstore_api", "viewpoint_get_all_removed_apps", group_1.$id );
	    log.debug( json.debug( apps ) );
	    expect( apps		).to.have.length( 1 );
	}

	const ma_state			= await clients.alice.call("appstore", "appstore_api", "get_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state.message	).to.equal( moderator_action.message );
    });

    it("should unremove app from group view", async function () {
	const updated_ma_entry			= await clients.alice.call("appstore", "appstore_api", "update_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	    "message": "Developer fixed the app",
	    "metadata": {
		"remove": false,
	    },
	});

	log.debug("Unremoved app: %s", json.debug(updated_ma_entry) );

	const apps			= await clients.alice.call("appstore", "appstore_api", "viewpoint_get_all_apps", group_1.$id );

	log.debug( json.debug( apps ) );

	expect( apps			).to.have.length( 1 );

	const ma_state			= await clients.alice.call("appstore", "appstore_api", "get_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state.message	).to.equal( updated_ma_entry.message );
    });

    it("should get moderator actions", async function () {
	const moderator_actions		= await clients.alice.call("appstore", "appstore_api", "get_moderator_actions", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	log.debug( json.debug( moderator_actions ) );
    });

}


const ICON_SIZE_LIMIT		= 204_800;

function errors_tests () {

    it("should fail to remove app because agent is not a group member", async function () {
	await expect_reject( async () => {
	    const moderator_action		= await clients.bobby.call("appstore", "appstore_api", "update_moderated_state", {
		"group_id": group_1.$id,
		"app_id": app_1.$id,
		"message": "malicious",
		"metadata": {
		    "remove": true,
		},
	    });
	}, "is not authorized to update content managed by group" );
    });

}

describe("Controlled Viewpoint", () => {
    const crux				= new CruxConfig();
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
    });

    before(async function () {
	this.timeout( 60_000 );

	const actors			= await holochain.backdrop({
	    "test_happ": {
		"appstore":	APPSTORE_DNA_PATH,
	    },
	}, {
	    "actors": [ "alice", "bobby" ],
	});

	for ( let name in actors ) {
	    for ( let app_prefix in actors[ name ] ) {
		log.info("Upgrade client for %s => %s", name, app_prefix );
		const client		= clients[ name ]	= actors[ name ][ app_prefix ].client;

		crux.upgrade( client );
	    }
	}

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.call( "appstore", "appstore_api", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
	{
	    let whoami			= await clients.bobby.call( "appstore", "appstore_api", "whoami", null, 30_000 );
	    log.normal("Bobby whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
    });

    describe("Publisher", publisher_tests.bind( this, holochain ) );
    describe("App", app_tests.bind( this, holochain ) );
    describe("Group Viewpoint", group_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});
