import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-multiple", process.env.LOG_LEVEL );

import * as fs				from 'node:fs/promises';
import path				from 'path';

import { expect }			from 'chai';
import { encode, decode }		from '@msgpack/msgpack';
import {
    Bundle,
}					from '@spartan-hc/bundles';
import {
    sha256,
}					from '../utils.js';


let src_bundle;
let src_happ_bundle;

const __dirname				= path.dirname( new URL(import.meta.url).pathname );
const APPSTORE_PATH			= path.join( __dirname, "../../happ/appstore.happ" );


describe("Repacking error", () => {

    it("should fail to repack the same", async function () {
	log.normal("Create test webhapp bundle");
	src_bundle				= Bundle.createWebhapp({
	    "name": "fake-webhapp-1",
	    "ui": {
		"bytes": new Uint8Array( Array( 1_000 ).fill( 1 ) ),
	    },
	    "happ_manifest": {
		"bytes": await fs.readFile( APPSTORE_PATH ),
	    },
	});

	// Setup source bundle for comparison later
	log.normal("Rebundle DNAs and hApp for consistent bytes");
	{
	    src_happ_bundle			= src_bundle.happ();

	    src_happ_bundle.dnas().forEach( (dna_bundle, i) => {
		const role_manifest		= src_happ_bundle.manifest.roles[i];
		const rpath			= role_manifest.dna.bundled;

		// Replace DNA bytes with deterministic bytes
		log.normal("Rebundle '%s' DNA for consistent bytes", role_manifest.name );
		src_happ_bundle.resources[ rpath ]	= dna_bundle.toBytes({ sortKeys: true });
	    });

	    {
		log.normal("Rebundle hApp for consistent bytes");
		const rpath			= src_bundle.manifest.happ_manifest.bundled;
		src_bundle
		    .resources[ rpath ]	= src_happ_bundle.toBytes({ sortKeys: true });
	    }
	}

	const src_happ_bytes		= src_bundle.resources[
	    src_bundle.manifest.happ_manifest.bundled
	];

	const src_happ_msgpack_bytes	= Bundle.gunzip( src_happ_bytes );
	const src_happ_repack_mp_bytes	= src_happ_bundle.toEncoded({ sortKeys: true });

	const src_happ_content		= Bundle.msgpackDecode( src_happ_msgpack_bytes );
	const src_happ_repack_content	= Bundle.msgpackDecode( src_happ_repack_mp_bytes );

	log.normal("Compare re-encoded msgpacks");
	expect(
	    sha256( encode( src_happ_content, { sortKeys: true } ) )
	).to.equal(
	    sha256( encode( src_happ_repack_content, { sortKeys: true } ) )
	);

	log.normal("Compare contents");
	expect(
	    src_happ_content
	).to.deep.equal(
	    src_happ_repack_content
	);

	log.normal("Compare msgpack bytes");
	expect(
	    sha256( src_happ_msgpack_bytes )
	).to.equal(
	    sha256( src_happ_repack_mp_bytes )
	);

	log.normal("Compare bundles");
	expect(
	    (new Bundle(src_happ_bytes))
	).to.deep.equal(
	    src_happ_bundle
	);
    });

})
