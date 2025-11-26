import path from 'node:path';
import {writeJsonFile, writeJsonFileSync} from 'write-json-file';
import {readPackage, readPackageSync} from 'read-pkg';
import {deepmerge} from 'deepmerge-ts';
import {sanitize, normalize, shouldDisableDetectIndent} from './util.js';

export async function updatePackage(filePath, data, options) {
	({filePath, data, options} = sanitize(filePath, data, options));

	let package_;

	try {
		package_ = await readPackage({cwd: path.dirname(filePath), normalize: false});
	} catch (error) {
		// 'package.json' doesn't exist
		if (error.code === 'ENOENT') {
			return writeJsonFile(filePath, data, options);
		}

		throw error;
	}

	// Disable detectIndent for empty packages to ensure proper formatting
	if (shouldDisableDetectIndent(package_, options)) {
		options = {...options, detectIndent: false};
	}

	package_ = deepmerge(package_, data);

	if (options.normalize) {
		package_ = normalize(package_);
	}

	return writeJsonFile(filePath, package_, options);
}

export function updatePackageSync(filePath, data, options) {
	({filePath, data, options} = sanitize(filePath, data, options));

	let package_;

	try {
		package_ = readPackageSync({cwd: path.dirname(filePath), normalize: false});
	} catch (error) {
		// 'package.json' doesn't exist
		if (error.code === 'ENOENT') {
			writeJsonFileSync(filePath, data, options);
			return;
		}

		throw error;
	}

	// Disable detectIndent for empty packages to ensure proper formatting
	if (shouldDisableDetectIndent(package_, options)) {
		options = {...options, detectIndent: false};
	}

	package_ = deepmerge(package_, data);

	if (options.normalize) {
		package_ = normalize(package_);
	}

	writeJsonFileSync(filePath, package_, options);
}
