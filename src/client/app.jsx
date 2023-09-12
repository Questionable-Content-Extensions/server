import React, { Component } from "react";

import s from "./screenshot.png";

export default class App extends Component {
    render() {
        return (
            <div>
                <h1>Welcome to Questionable Content Extensions</h1>
                <p>
                    Questionable Content Extensions is a project to add
                    additional features to the{" "}
                    <a href="http://questionablecontent.net/">
                        Questionable Content <i class="fa fa-external-link"></i>
                    </a>{" "}
                    comic.
                </p>
                <p>
                    For now, the only extension made is a user script for Chrome
                    (requires{" "}
                    <a href="https://chrome.google.com/webstore/detail/tampermonkey/dhdgffkkebhmkfjojejmpbldmpobfkfo?hl=en">
                        Tampermonkey <i class="fa fa-external-link"></i>
                    </a>{" "}
                    or equivalent) and Firefox (requires{" "}
                    <a href="https://addons.mozilla.org/en-US/firefox/addon/greasemonkey/">
                        Greasemonkey <i class="fa fa-external-link"></i>
                    </a>{" "}
                    or equivalent) (untested in other browsers). You can always
                    find{" "}
                    <a href="/releases/qc-ext.latest.user.js">
                        the latest version of the script
                    </a>{" "}
                    right here.
                </p>
                <p>
                    The source code for the script and its issue tracker can be
                    found{" "}
                    <a href="https://github.com/Questionable-Content-Extensions/client">
                        on its GitHub project page{" "}
                        <i class="fa fa-external-link"></i>
                    </a>
                    .
                </p>
                <p>
                    Finally, for broader discussions about the extension, please
                    come visit{" "}
                    <a href="https://www.reddit.com/r/questionablextensions/">
                        the subreddit <i class="fa fa-external-link"></i>
                    </a>
                    .
                </p>
                <h2>Screenshot</h2>
                <img src={s} alt="Screenshot of the userscript in action" />
            </div>
        );
    }
}
