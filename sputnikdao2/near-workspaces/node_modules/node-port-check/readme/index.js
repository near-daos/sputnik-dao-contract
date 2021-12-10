let fs = require('fs');
let readline = require('readline');
let semver = require('semver');
let root = function(){

    let path = require('path');
    let _root = path.resolve(__dirname);
    let input = Array.prototype.slice.call(arguments, 0);
    return path.join.apply(path, [_root].concat(input));

}
let currentVersion = require(root('../package.json')).version;
let readmeFile = root('..', 'README.md');

const versionArgument = typeof process.argv[2] === 'undefined' ? 'patch' : process.argv[2];

currentVersion = semver.inc(currentVersion, versionArgument);

const EOL = require("os").EOL;

let markdown = "";

fs.readFile(root('..', 'readme/shared/header.md'), {encoding: 'utf8'}, (err, data) => {

    markdown += data;

    fs.readFile(root('..', 'readme/shared/content.md'), {encoding: 'utf8'}, (err, data) => {

        let appDescription = data;

        fs.readFile(root('..', 'readme/shared/history.md'), {encoding: 'utf8'}, (err, data) => {

            let history = data;

            let nextVersion = "";

            let rd = readline.createInterface(fs.createReadStream(root('..', 'readme/nextVersion.txt')));

            rd.on('line', (input) => {

                input = input.trim();

                if (input.length) {

                    nextVersion += "- " + input.substr(0, 1).toUpperCase() + input.substr(1) + EOL;

                }

            }).on('close', () => {

                nextVersion = nextVersion.trim();

                if (!nextVersion.length) {

                    console.log(`No data received for README.md`);

                } else {

                    history = `${currentVersion} :
----------------
${nextVersion}${EOL}
${history}`;

                    markdown += `${EOL}# Update${EOL}${EOL}${history}${EOL}${EOL}`;

                    markdown += appDescription;


                    fs.writeFile(readmeFile, markdown, {encoding: "utf8"}, (err, data) => {

                        if (err) {

                            console.log("Could not create README.md");

                        } else {

                            fs.writeFileSync(root('..', 'readme/nextVersion.txt'), "", {encoding: "utf8"});

                            console.log(`README.md has been generated for ${currentVersion}`);

                            fs.writeFile(root('..', 'readme/shared/history.md'), history, {encoding: "utf8"}, (err, data) => {

                                if (err) {
                                    console.log("Could not update history.md");
                                } else {
                                    console.log("history.md has been updated");
                                }

                            });

                        }

                    });

                }

            })

        });

    });

});
