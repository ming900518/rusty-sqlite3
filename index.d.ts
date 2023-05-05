/**
 * @param {string} url
 * @returns {Promise<boolean>}
 */
export function connect(url: string): Promise<boolean>;

/**
 * @template T
 * @param {string} sql
 * @param {any[]} [args]
 * @returns {Promise.<T[]>}
 */
export function execute<T>(sql: string, args?: any[]): Promise<T[]>;
