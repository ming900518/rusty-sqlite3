/**
 * @param {string} url
 * @returns {boolean}
 */
export function connect(url: string): boolean;

/**
 * @param {string} sql
 * @param {any[]} [args]
 * @returns {T[]}
 */
export function execute<T>(sql: string, args?: any[]): T[];
