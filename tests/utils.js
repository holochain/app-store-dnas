
import crypto				from 'crypto';
import { expect }			from 'chai';
import {
    HoloHash,
    DnaHash, AgentPubKey,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash';


export async function expect_reject ( cb, error, message ) {
    let failed				= false;
    try {
	await cb();
    } catch (err) {
	failed				= true;
	expect( () => { throw err }	).to.throw( error, message );
    }
    expect( failed			).to.be.true;
}


export function linearSuite ( name, setup_fn, args_fn ) {
    describe( name, function () {
	beforeEach(function () {
	    let parent_suite		= this.currentTest.parent;
	    if ( parent_suite.tests.some(test => test.state === "failed") )
		this.skip();
	    if ( parent_suite.parent?.tests.some(test => test.state === "failed") )
		this.skip();
	});
	setup_fn.call( this, args_fn );
    });
}


export function createPublisherInput ( overrides ) {
    return Object.assign({
	"name": "Holo",
	"location": "Gibraltar",
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


export function createAppInput ( overrides ) {
    return Object.assign({
	"title": "Chess",
	"subtitle": "The classic boardgame",
	"description": "The boardgame known as Chess",
	"icon": crypto.randomBytes(1_000),
	"publisher": new ActionHash( crypto.randomBytes(32) ),
	"apphub_hrl": {
	    "dna": new DnaHash( crypto.randomBytes(32) ),
	    "target": new ActionHash( crypto.randomBytes(32) ),
	},
	"apphub_hrl_hash": new EntryHash( crypto.randomBytes(32) ),
	"editors": [
	    new AgentPubKey( crypto.randomBytes(32) )
	],
    }, overrides );
};


export function createAppVersionInput ( overrides ) {
    return Object.assign({
	"version": "0.1.0",
	"for_app": new ActionHash( crypto.randomBytes(32) ),
	"apphub_hrl": {
	    "dna": new DnaHash( crypto.randomBytes(32) ),
	    "target": new ActionHash( crypto.randomBytes(32) ),
	},
	"apphub_hrl_hash": new EntryHash( crypto.randomBytes(32) ),
    }, overrides );
};


export function createGroupInput ( admins, ...members ) {
    return {
	"admins": admins,
	"members": [ ...members ],

	"published_at":		Date.now(),
	"last_updated":		Date.now(),
	"metadata":		{},
    };
};

export function sha256 ( bytes ) {
    const hash				= crypto.createHash("sha256");

    hash.update( bytes );

    return hash.digest("hex");
}


export default {
    expect_reject,
    linearSuite,
    createAppInput,
    createAppVersionInput,
    createPublisherInput,
    createGroupInput,
    sha256,
};
