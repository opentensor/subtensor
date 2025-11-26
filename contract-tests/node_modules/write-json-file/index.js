import path from 'node:path';
import fs, {promises as fsPromises} from 'node:fs';
import writeFileAtomic from 'write-file-atomic';
import sortKeys from 'sort-keys';
import detectIndent from 'detect-indent';
import isPlainObj from 'is-plain-obj';

const init = (function_, filePath, data, options) => {
	if (!filePath) {
		throw new TypeError('Expected a filepath');
	}

	if (data === undefined) {
		throw new TypeError('Expected data to stringify');
	}

	options = {
		indent: '\t',
		sortKeys: false,
		...options,
	};

	if (options.sortKeys && isPlainObj(data)) {
		data = sortKeys(data, {
			deep: true,
			compare: typeof options.sortKeys === 'function' ? options.sortKeys : undefined,
		});
	}

	return function_(filePath, data, options);
};

const main = async (filePath, data, options) => {
	let {indent} = options;
	let trailingNewline = '\n';
	try {
		const file = await fsPromises.readFile(filePath, 'utf8');
		if (!file.endsWith('\n')) {
			trailingNewline = '';
		}

		if (options.detectIndent) {
			indent = detectIndent(file).indent;
		}
	} catch (error) {
		if (error.code !== 'ENOENT') {
			throw error;
		}
	}

	const json = JSON.stringify(data, options.replacer, indent);

	return writeFileAtomic(filePath, `${json}${trailingNewline}`, {mode: options.mode, chown: false});
};

const mainSync = (filePath, data, options) => {
	let {indent} = options;
	let trailingNewline = '\n';
	try {
		const file = fs.readFileSync(filePath, 'utf8');
		if (!file.endsWith('\n')) {
			trailingNewline = '';
		}

		if (options.detectIndent) {
			indent = detectIndent(file).indent;
		}
	} catch (error) {
		if (error.code !== 'ENOENT') {
			throw error;
		}
	}

	const json = JSON.stringify(data, options.replacer, indent);

	return writeFileAtomic.sync(filePath, `${json}${trailingNewline}`, {mode: options.mode, chown: false});
};

export async function writeJsonFile(filePath, data, options) {
	await fsPromises.mkdir(path.dirname(filePath), {recursive: true});
	await init(main, filePath, data, options);
}

export function writeJsonFileSync(filePath, data, options) {
	fs.mkdirSync(path.dirname(filePath), {recursive: true});
	init(mainSync, filePath, data, options);
}
