
import {
    HoloHash, AnyDhtHash,
    AgentPubKey, DnaHash,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash';
import {
    ScopedEntity,
    intoStruct,
    AnyType, OptionType,
    VecType, MapType,
}					from '@spartan-hc/caps-entities';



export const WebAddressStruct = {
    "url":			String,
    "context":			OptionType( String ),
};

export const DeprecationNoticeStruct = {
    "message":			String,
    "recommended_alternatives":	OptionType( VecType( ActionHash ) ),
};

export const HRLStruct = {
    "dna":			DnaHash,
    "target":			AnyDhtHash,
}

export const GroupRefStruct = [ ActionHash, ActionHash ];



export const PublisherStruct = {
    "name":			String,
    "location":			String,
    "website":			WebAddressStruct,
    "editors_group_id":		GroupRefStruct,

    // common fields
    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			MapType( String, AnyType ),

    // optional
    "description":		OptionType( String ),
    "email":			OptionType( String ),
    "icon":			OptionType( EntryHash ),
    "deprecation":		OptionType( DeprecationNoticeStruct ),
};

export function PublisherEntry ( entry ) {
    return intoStruct( entry, PublisherStruct );
}

export class Publisher extends ScopedEntity {
    static STRUCT		= PublisherStruct;

    async $refresh () {
	const result		= await this.zome.get_publisher( this.$id );

	super.$update( result );
    }

    async $update ( changes ) {
	const result		= await this.zome.update_publisher({
	    "base": this.$action,
	    "properties": changes,
	});

	super.$update( result );

	return this;
    }

    async $deprecate ( message ) {
	const result		= await this.zome.deprecate_publisher({
	    "base": this.$action,
	    message,
	});

	super.$update( result );

	return this;
    }

    async $undeprecate () {
	const result		= await this.zome.undeprecate_publisher({
	    "base": this.$action,
	});

	super.$update( result );

	return this;
    }

}



export const AppStruct = {
    "title":			String,
    "subtitle":			String,
    "description":		String,
    "publisher":		ActionHash,
    "apphub_hrl":		HRLStruct,
    "apphub_hrl_hash":		EntryHash,
    "editors":			VecType( AgentPubKey ),

    // common fields
    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			MapType( String, AnyType ),

    // optional
    "icon":			OptionType( EntryHash ),
    "deprecation":		OptionType( DeprecationNoticeStruct ),
};

export function AppEntry ( entry ) {
    return intoStruct( entry, AppStruct );
}

export class App extends ScopedEntity {
    static STRUCT		= AppStruct;

    async $update ( changes ) {
	const result		= await this.zome.update_app({
	    "base": this.$action,
	    "properties": changes,
	});

	super.$update( result );

	return this;
    }

    async $getVersions () {
	return await this.zome.get_app_versions_for_app({
	    "for_app":		this.$id,
	});
    }

    async $getLatestVersion () {
	const versions		= await this.$getVersions();

	if ( versions.length === 0 )
	    throw new Error(`There are no versions for App (${this.$id})`);

	return versions[0];
    }

    async $getModeratedState ( group_id ) {
	return await this.zome.get_moderated_state({
	    group_id,
	    "app_id": this.$id,
	});
    }

    async $updateModeratedState ( group_id, input ) {
	return await this.zome.update_moderated_state({
	    group_id,
	    "app_id": this.$id,
	    "message": input.message,
	    "metadata": input.metadata,
	});
    }

    async $deprecate ( message ) {
	const result		= await this.zome.deprecate_app({
	    "base": this.$action,
	    message,
	});

	super.$update( result );

	return this;
    }

    async $undeprecate () {
	const result		= await this.zome.undeprecate_app({
	    "base": this.$action,
	});

	super.$update( result );

	return this;
    }

    //
    // "apphub" methods
    //
    async $getWebAppPackage () {
	return await this.zome.get_apphub_webapp_package({
	    "dna":		this.apphub_hrl.dna,
	    "target":		this.apphub_hrl.target,
	    "hash":		this.apphub_hrl_hash,
	});
    }

    async $getWebAppPackageVersions () {
	return await this.zome.get_apphub_webapp_package_versions( this.$id );
    }

}



export const BundleHashesStruct = {
    "hash":			String,
    "ui_hash":			String,
    "happ_hash":		String,
};

export const AppVersionStruct = {
    "version":			String,
    "for_app":			ActionHash,
    "apphub_hrl":		HRLStruct,
    "apphub_hrl_hash":		EntryHash,
    "bundle_hashes":		BundleHashesStruct,

    // common fields
    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			MapType( String, AnyType ),
};

export function AppVersionEntry ( entry ) {
    return intoStruct( entry, AppVersionStruct );
}

export class AppVersion extends ScopedEntity {
    static STRUCT		= AppVersionStruct;

    //
    // "apphub" methods
    //
    async $getWebAppPackageVersion () {
	return await this.zome.get_apphub_webapp_package_version({
	    "dna":		this.apphub_hrl.dna,
	    "target":		this.apphub_hrl.target,
	    "hash":		this.apphub_hrl_hash,
	});
    }

    async $getBundle () {
	return await this.zome.get_apphub_webapp_package_version_bundle( this.$id );
    }
}



export const GroupStruct = {
    "admins":			VecType( AgentPubKey ),
    "members":			VecType( AgentPubKey ),
    "deleted":			OptionType( Boolean ),

    // common fields
    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			MapType( String, AnyType ),
};

export function GroupEntry ( entry ) {
    return intoStruct( entry, GroupStruct );
}

export class Group extends ScopedEntity {
    static STRUCT		= GroupStruct;

    async $getAllApps () {
	return await this.zome.viewpoint_get_all_apps( this.$id );
    }

    async $getAllRemovedApps () {
	return await this.zome.viewpoint_get_all_removed_apps( this.$id );
    }

    async $getAppModeratedState ( app_id ) {
	return await this.zome.get_moderated_state({
	    "group_id": this.$id,
	    app_id,
	});
    }

    async $getAppModeratedActions ( app_id ) {
	return await this.zome.get_moderator_actions({
	    "group_id": this.$id,
	    app_id,
	});
    }

    async $removeApp ( app_id, message ) {
	const ma_state		= await this.$getAppModeratedState( app_id );
	const metadata		= Object.assign( {}, ma_state?.metastate, {
	    "remove": true,
	});

	return await this.zome.update_moderated_state({
	    "group_id": this.$id,
	    app_id,
	    message,
	    metadata,
	});
    }

    async $unremoveApp ( app_id, message ) {
	const ma_state		= await this.$getAppModeratedState( app_id );
	const metadata		= Object.assign( {}, ma_state?.metastate, {
	    "remove": false,
	});

	return await this.zome.update_moderated_state({
	    "group_id": this.$id,
	    app_id,
	    message,
	    metadata,
	});
    }
}



export default {
    PublisherStruct,
    PublisherEntry,
    Publisher,

    AppStruct,
    AppEntry,
    App,

    AppVersionStruct,
    AppVersionEntry,
    AppVersion,

    GroupStruct,
    GroupEntry,
    Group,
};
