import {promises as fs, readFileSync} from 'node:fs';
import {writeJsonFile, writeJsonFileSync} from 'write-json-file';
import {sanitize, shouldDisableDetectIndent} from './util.js';

export async function writePackage(filePath, data, options) {
	({filePath, data, options} = sanitize(filePath, data, options));

	// Disable detectIndent for empty files to ensure proper formatting
	try {
		const content = await fs.readFile(filePath, 'utf8');
		if (shouldDisableDetectIndent(content, options)) {
			options = {...options, detectIndent: false};
		}
	} catch {
		// File doesn't exist, no special handling needed
	}

	return writeJsonFile(filePath, data, options);
}

export function writePackageSync(filePath, data, options) {
	({filePath, data, options} = sanitize(filePath, data, options));

	// Disable detectIndent for empty files to ensure proper formatting
	try {
		const content = readFileSync(filePath, 'utf8');
		if (shouldDisableDetectIndent(content, options)) {
			options = {...options, detectIndent: false};
		}
	} catch {
		// File doesn't exist, no special handling needed
	}

	writeJsonFileSync(filePath, data, options);
}
