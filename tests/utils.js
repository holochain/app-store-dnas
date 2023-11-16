
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


export async function createPublisherInput ( mm_zomelet, overrides ) {
    const input			= Object.assign({
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

    input.icon			= await mm_zomelet.save( input.icon );

    return input;
};


const TEST_DNA_HASH			= "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5";

export async function createAppInput ( mm_zomelet, overrides ) {
    const input			= Object.assign({
	"title": "Chess",
	"subtitle": "The classic boardgame",
	"description": "The boardgame known as Chess",
	"icon": crypto.randomBytes(1_000),
	"publisher": new ActionHash( crypto.randomBytes(32) ),
	"devhub_address": {
	    "dna": TEST_DNA_HASH,
	    "happ": new ActionHash( crypto.randomBytes(32) ),
	    "gui": new ActionHash( crypto.randomBytes(32) ),
	},
	"editors": [
	    new AgentPubKey( crypto.randomBytes(32) )
	],
    }, overrides );

    input.icon			= await mm_zomelet.save( input.icon );

    return input;
};


export default {
    expect_reject,
    linearSuite,
    createAppInput,
    createPublisherInput,
};
