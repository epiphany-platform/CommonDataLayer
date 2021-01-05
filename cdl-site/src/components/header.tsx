import { FunctionalComponent, h } from "preact";
import { Link } from "preact-router/match";

interface HeaderProps {
    menuOpen: boolean;
    toggleMenu: (isOpen: boolean) => void;
}

const Header: FunctionalComponent<HeaderProps> = ({ menuOpen, toggleMenu }) => (
    <nav>
        <div class="nav-container">
            <div class="nav-logo">
                <Link activeClassName="active" href="/">Common Data Layer</Link>
            </div>
            <ul class="nav-links">
                <li><Link activeClassName="active" href="/insert">Insert</Link></li>
                <li><Link activeClassName="active" href="/query">Query</Link></li>
                <li><Link activeClassName="active" href="/schemas">Schemas</Link></li>
                <li><Link activeClassName="active" href="/settings">Settings</Link></li>
            </ul>
            <a class="mobile-menu-toggle" onClick={(event) => {
                event.preventDefault();
                event.stopPropagation();
                toggleMenu(!menuOpen);
            }} />
            <ul class="mobile-menu menu" style={{ display: menuOpen ? "block" : "none" }}>
                <li><a href="/insert">Insert Data</a></li>
                <li><a href="/query">Query Data</a></li>
                <li><a href="/schemas">Schemas</a></li>
                <li><a href="/settings">Settings</a></li>
            </ul>
        </div>
    </nav>
);

export default Header;
