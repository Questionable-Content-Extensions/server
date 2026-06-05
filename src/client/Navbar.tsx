import { useState } from 'react';

import ExternalLinkIcon from './ExternalLinkIcon';

interface NavLinkProps {
    href: string;
    external?: boolean;
    children: React.ReactNode;
    className?: string;
}

function NavLink({ href, external, children, className }: NavLinkProps) {
    return (
        <a href={href} className={className}>
            {children}
            {external && <ExternalLinkIcon />}
        </a>
    );
}

export default function Navbar() {
    const [menuOpen, setMenuOpen] = useState(false);

    const desktopLinkClass =
        'text-gray-300 hover:text-white px-3 py-2 rounded-md text-sm font-medium inline-flex items-center gap-1';
    const mobileLinkClass =
        'text-gray-300 hover:text-white px-3 py-2 rounded-md text-base font-medium inline-flex items-center gap-1';

    return (
        <nav className="bg-gray-900 fixed top-0 left-0 right-0 z-50">
            <div className="mx-auto max-w-7xl px-4">
                <div className="flex items-center justify-between h-14">
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            className="md:hidden inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-700 focus:outline-none"
                            aria-expanded={menuOpen}
                            aria-controls="navbar"
                            onClick={() => setMenuOpen((o) => !o)}
                        >
                            <span className="sr-only">Toggle navigation</span>
                            <svg
                                className="h-6 w-6"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                aria-hidden="true"
                            >
                                <path
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth={2}
                                    d="M4 6h16M4 12h16M4 18h16"
                                />
                            </svg>
                        </button>
                        <a className="text-white font-bold text-lg" href="/">
                            Questionable Content Extensions
                        </a>
                    </div>
                    <ul className="hidden md:flex items-center">
                        <li>
                            <NavLink href="/" className={desktopLinkClass}>
                                Home
                            </NavLink>
                        </li>
                        <li>
                            <NavLink
                                href="/releases/qc-ext.latest.user.js"
                                className={desktopLinkClass}
                            >
                                User script
                            </NavLink>
                        </li>
                        <li>
                            <NavLink
                                href="https://github.com/Questionable-Content-Extensions/client"
                                external
                                className={desktopLinkClass}
                            >
                                User script GitHub project
                            </NavLink>
                        </li>
                        <li>
                            <NavLink
                                href="https://www.reddit.com/r/questionablextensions/"
                                external
                                className={desktopLinkClass}
                            >
                                Subreddit
                            </NavLink>
                        </li>
                    </ul>
                </div>
                {menuOpen && (
                    <div id="navbar">
                        <ul className="px-2 pt-2 pb-3 space-y-1">
                            <li>
                                <NavLink href="/" className={mobileLinkClass}>
                                    Home
                                </NavLink>
                            </li>
                            <li>
                                <NavLink
                                    href="/releases/qc-ext.latest.user.js"
                                    className={mobileLinkClass}
                                >
                                    User script
                                </NavLink>
                            </li>
                            <li>
                                <NavLink
                                    href="https://github.com/Questionable-Content-Extensions/client"
                                    external
                                    className={mobileLinkClass}
                                >
                                    User script GitHub project
                                </NavLink>
                            </li>
                            <li>
                                <NavLink
                                    href="https://www.reddit.com/r/questionablextensions/"
                                    external
                                    className={mobileLinkClass}
                                >
                                    Subreddit
                                </NavLink>
                            </li>
                        </ul>
                    </div>
                )}
            </div>
        </nav>
    );
}
