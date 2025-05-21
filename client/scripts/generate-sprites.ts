import svgSprite from 'svg-sprite';
import { mkdirSync, writeFileSync, readFileSync } from 'fs';
import { resolve, basename, dirname } from 'path';
import { sync } from 'glob';

const config = {
	mode: {
		symbol: {
			dest: 'static',
			sprite: 'sprite.svg'
		}
	},
	shape: {
		id: {
			generator: (name) => {
				return basename(name, '.svg');
			}
		}
	},
	svg: {
		xmlDeclaration: false,
		doctypeDeclaration: false
	}
};

const spriter = new svgSprite(config);

const svgFiles = sync('src/lib/assets/icons/*.svg');

svgFiles.forEach((file) => {
	spriter.add(resolve(file), basename(file), readFileSync(file, 'utf-8'));
});

// Compile the sprite
spriter.compile((error, result) => {
	if (error) {
		console.error(`Error generating sprite: ${error}`);
		return;
	}

	// Write the sprite file
	for (const mode in result) {
		for (const resource in result[mode]) {
			mkdirSync(dirname(result[mode][resource].path), { recursive: true });
			writeFileSync(result[mode][resource].path, result[mode][resource].contents);
		}
	}

	console.log('SVG sprite generated successfully!');
});
