/**
 * Represents a UCI section object with its options and metadata
 */
interface SectionObject {
    /** Indicates if the section is anonymous */
    '.anonymous': boolean;

    /** Sort order index of the section */
    '.index': number;

    /** Name or ID of the section */
    '.name': string;

    /** Type of the UCI section */
    '.type': string;

    /** Optional creation name for new sections */
    '.create'?: string;

    /** Additional UCI options as string or string array values */
    [key: string]: boolean | number | string | string[];
}

/**
 * Callback for section enumeration
 */
type SectionCallback = (section: SectionObject, sid: string) => void;

/**
 * Change record format returned by uci.changes()
 */
type ChangeRecord = [
    /** Operation name (add, set, remove, order, list-add, list-del, rename) */
    string,

    /** Target section ID */
    string,

    /** Operation-specific argument (type, option name, sort index) */
    string,

    /** Operation-specific value (option value, new name) */
    string?
];

class Uci {
    /**
     * Creates a new UCI instance
     */
    constructor();

    /**
     * Generates a new, unique section ID for the given configuration
     *
     * @param config - The configuration to generate the ID for
     * @returns A newly generated section ID in the form "newXXXXXX"
     */
    createSID(config: string): string;

    /**
     * Resolves a section ID in extended notation to its internal value
     *
     * @param config - The configuration to resolve the ID in
     * @param sid - The section ID to resolve
     * @returns The resolved section ID or null if not found
     */
    resolveSID(config: string, sid: string): string | null;

    /**
     * Loads UCI configurations from the remote ubus API
     *
     * @param config - Configuration name(s) to load
     * @returns Promise resolving to loaded configuration names
     */
    load(config: string | string[]): Promise<string[]>;

    /**
     * Unloads UCI configurations from the local cache
     *
     * @param config - Configuration name(s) to unload
     */
    unload(config: string | string[]): void;

    /**
     * Adds a new section to a configuration
     *
     * @param config - Configuration to add to
     * @param type - Type of section to add
     * @param name - Optional name for the section
     * @returns ID of the newly added section
     */
    add(config: string, type: string, name?: string): string;

    /**
     * Removes a section from a configuration
     *
     * @param config - Configuration to remove from
     * @param sid - ID of section to remove
     */
    remove(config: string, sid: string): void;

    /**
     * Enumerates sections in a configuration
     *
     * @param config - Configuration to enumerate
     * @param type - Optional section type filter
     * @param callback - Optional callback for each section
     * @returns Array of section objects
     */
    sections(config: string, type?: string, callback?: SectionCallback): SectionObject[];

    /**
     * Gets a section or option value
     *
     * @param config - Configuration to read from
     * @param sid - Section ID to read
     * @param option - Optional option name to read
     * @returns Section object or option value
     */
    get(config: string, sid: string, option?: string): null | string | string[] | SectionObject;

    /**
     * Sets an option value
     *
     * @param config - Configuration to modify
     * @param sid - Section ID to modify
     * @param option - Option name to set
     * @param value - Value to set
     */
    set(config: string, sid: string, option: string, value: null | string | string[]): void;

    /**
     * Removes an option
     *
     * @param config - Configuration to modify
     * @param sid - Section ID to modify
     * @param option - Option name to remove
     */
    unset(config: string, sid: string, option: string): void;

    /**
     * Gets the first matching section or option
     *
     * @param config - Configuration to read from
     * @param type - Optional section type filter
     * @param option - Optional option name to read
     * @returns Section object or option value
     */
    get_first(config: string, type?: string, option?: string): null | string | string[] | SectionObject;

    /**
     * Sets an option in the first matching section
     *
     * @param config - Configuration to modify
     * @param type - Optional section type filter
     * @param option - Option name to set
     * @param value - Value to set
     */
    set_first(config: string, type?: string, option: string, value: null | string | string[]): void;

    /**
     * Removes an option from the first matching section
     *
     * @param config - Configuration to modify
     * @param type - Optional section type filter
     * @param option - Option name to remove
     */
    unset_first(config: string, type?: string, option: string): void;

    /**
     * Moves a section before or after another section
     *
     * @param config - Configuration to modify
     * @param sid1 - ID of section to move
     * @param sid2 - Target section ID
     * @param after - Insert after target instead of before
     * @returns True if move succeeded
     */
    move(config: string, sid1: string, sid2: string | null, after?: boolean): boolean;

    /**
     * Saves all local changes to the remote API
     *
     * @returns Promise resolving to reloaded config names
     */
    save(): Promise<string[]>;

    /**
     * Applies saved changes with rollback protection
     *
     * @param timeout - Confirmation timeout in seconds
     * @returns Promise resolving to ubus status code
     */
    apply(timeout?: number): Promise<number>;

    /**
     * Fetches uncommitted changes from remote API
     *
     * @returns Promise resolving to change records by config
     */
    changes(): Promise<Record<string, ChangeRecord[]>>;
}

export const uci: Uci;
