import { FunctionalComponent, h } from "preact";
import { Link } from "preact-router/match";

const NotFound: FunctionalComponent = () => (
    <div class="container container-small">
        <div class="row">
            <div class="col align-center">
                <h2>Error 404</h2>
                <p>That page doesn't exist.</p>
                <Link href="/"><button>Back to Home</button></Link>
            </div>
        </div>
    </div>
);

export default NotFound;
