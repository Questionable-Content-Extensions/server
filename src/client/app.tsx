import { Component } from 'react';

import s from './screenshot.png';

function ExternalLinkIcon() {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            className="inline w-3.5 h-3.5 ml-0.5 align-text-top"
            aria-hidden="true"
        >
            <path
                fillRule="evenodd"
                d="M4.25 5.5a.75.75 0 00-.75.75v8.5c0 .414.336.75.75.75h8.5a.75.75 0 00.75-.75v-4a.75.75 0 011.5 0v4A2.25 2.25 0 0112.75 17h-8.5A2.25 2.25 0 012 14.75v-8.5A2.25 2.25 0 014.25 4h5a.75.75 0 010 1.5h-5z"
                clipRule="evenodd"
            />
            <path
                fillRule="evenodd"
                d="M6.194 12.753a.75.75 0 001.06.053L16.5 4.44v2.81a.75.75 0 001.5 0v-4.5a.75.75 0 00-.75-.75h-4.5a.75.75 0 000 1.5h2.553l-9.056 8.194a.75.75 0 00-.053 1.06z"
                clipRule="evenodd"
            />
        </svg>
    );
}

export default class App extends Component {
    render() {
        return (
            <div className="prose max-w-none py-6">
                <h1>Welcome to Questionable Content Extensions</h1>
                <p>
                    Questionable Content Extensions is a project to add
                    additional features to the{' '}
                    <a href="http://questionablecontent.net/">
                        Questionable Content <ExternalLinkIcon />
                    </a>{' '}
                    comic.
                </p>
                <p>
                    For now, the only extension made is a user script for Chrome
                    (requires{' '}
                    <a href="https://chrome.google.com/webstore/detail/tampermonkey/dhdgffkkebhmkfjojejmpbldmpobfkfo?hl=en">
                        Tampermonkey <ExternalLinkIcon />
                    </a>{' '}
                    or equivalent) and Firefox (requires{' '}
                    <a href="https://addons.mozilla.org/en-US/firefox/addon/greasemonkey/">
                        Greasemonkey <ExternalLinkIcon />
                    </a>{' '}
                    or equivalent) (untested in other browsers). You can always
                    find{' '}
                    <a href="/releases/qc-ext.latest.user.js">
                        the latest version of the script
                    </a>{' '}
                    right here.
                </p>
                <p>
                    The source code for the script and its issue tracker can be
                    found{' '}
                    <a href="https://github.com/Questionable-Content-Extensions/client">
                        on its GitHub project page <ExternalLinkIcon />
                    </a>
                    .
                </p>
                <p>
                    Finally, for broader discussions about the extension, please
                    come visit{' '}
                    <a href="https://www.reddit.com/r/questionablextensions/">
                        the subreddit <ExternalLinkIcon />
                    </a>
                    .
                </p>
                <h2>Screenshot</h2>
                <img src={s} alt="Screenshot of the userscript in action" />
            </div>
        );
    }
}
