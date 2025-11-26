import path from 'node:path';
import sortKeys from 'sort-keys';

export const dependencyKeys = new Set([
	'dependencies',
	'devDependencies',
	'optionalDependencies',
	'peerDependencies',
]);

export const hasMultipleDependencyTypes = dependencies => Object.keys(dependencies).some(key => dependencyKeys.has(key));

export function normalize(packageJson) {
	const result = {};

	for (const key of Object.keys(packageJson)) {
		if (!dependencyKeys.has(key)) {
			result[key] = packageJson[key];
		} else if (Object.keys(packageJson[key]).length > 0) {
			result[key] = sortKeys(packageJson[key]);
		}
	}

	return result;
}

export function sanitize(filePath, data, options, {sanitizeData = true} = {}) {
	if (typeof filePath !== 'string') {
		options = data;
		data = filePath;
		filePath = '.';
	}

	options = {
		normalize: true,
		detectIndent: true,
		indent: '\t',
		...options,
	};

	filePath = path.basename(filePath) === 'package.json' ? filePath : path.join(filePath, 'package.json');

	if (options.normalize && sanitizeData) {
		data = normalize(data);
	}

	return {filePath, data, options};
}

export function isEmptyPackage(packageContent) {
	return typeof packageContent === 'string'
		? packageContent.trim() === '{}'
		: Object.keys(packageContent).length === 0;
}

export function shouldDisableDetectIndent(contentOrPackage, options) {
	return isEmptyPackage(contentOrPackage) && options.detectIndent !== false;
}
