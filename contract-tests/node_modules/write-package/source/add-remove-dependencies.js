import path from 'node:path';
import {writeJsonFile, writeJsonFileSync} from 'write-json-file';
import {readPackage, readPackageSync} from 'read-pkg';
import {updatePackage, updatePackageSync} from './update-package.js';
import {sanitize, normalize, hasMultipleDependencyTypes} from './util.js';

export async function addPackageDependencies(filePath, dependencies, options) {
	return hasMultipleDependencyTypes(typeof filePath === 'string' ? dependencies : filePath)
		? updatePackage(filePath, {...dependencies}, options)
		: updatePackage(filePath, {dependencies}, options);
}

export function addPackageDependenciesSync(filePath, dependencies, options) {
	return hasMultipleDependencyTypes(typeof filePath === 'string' ? dependencies : filePath)
		? updatePackageSync(filePath, {...dependencies}, options)
		: updatePackageSync(filePath, {dependencies}, options);
}

export async function removePackageDependencies(filePath, dependencies, options) {
	({filePath, data: dependencies, options} = sanitize(filePath, dependencies, options, {sanitizeData: false}));

	let package_;

	try {
		package_ = await readPackage({cwd: path.dirname(filePath), normalize: false});
	} catch (error) {
		// 'package.json' doesn't exist
		if (error.code === 'ENOENT') {
			return;
		}

		throw error;
	}

	if (Array.isArray(dependencies)) {
		for (const dependency of dependencies) {
			delete package_.dependencies[dependency];
		}
	} else {
		for (const [dependencyKey, _dependencies] of Object.entries(dependencies)) {
			for (const dependency of _dependencies) {
				delete package_[dependencyKey][dependency];
			}
		}
	}

	if (options.normalize) {
		package_ = normalize(package_);
	}

	return writeJsonFile(filePath, package_, options);
}

export function removePackageDependenciesSync(filePath, dependencies, options) {
	({filePath, data: dependencies, options} = sanitize(filePath, dependencies, options, {sanitizeData: false}));

	let package_;

	try {
		package_ = readPackageSync({cwd: path.dirname(filePath), normalize: false});
	} catch (error) {
		// 'package.json' doesn't exist
		if (error.code === 'ENOENT') {
			return;
		}

		throw error;
	}

	if (Array.isArray(dependencies)) {
		for (const dependency of dependencies) {
			delete package_.dependencies[dependency];
		}
	} else {
		for (const [dependencyKey, _dependencies] of Object.entries(dependencies)) {
			for (const dependency of _dependencies) {
				delete package_[dependencyKey][dependency];
			}
		}
	}

	if (options.normalize) {
		package_ = normalize(package_);
	}

	writeJsonFileSync(filePath, package_, options);
}
