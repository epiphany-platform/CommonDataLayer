import { FunctionalComponent, ComponentProps, h } from "preact";
import { Link } from "preact-router/match";

import { RemoteData } from "../models";

interface RemoteContentProps<T> {
    data: RemoteData<T>,
    render: (data: T) => string;
}

export const RemoteContent = <T extends any>({ data, render }: RemoteContentProps<T>) => {
    if (data.status === "not-loaded") {
        return (<span></span>);
    } else if (data.status === "loading") {
        return (<span>loading...</span>);
    } else if (data.status === "error") {
        return (<ErrorBox error={data.error} />);
    } else {
        return render(data.data);
    }
};

export const ErrorBox: FunctionalComponent<{ error: string; }> = ({ error }) => (
    <p class="alert alert-warning">{error}</p>
);
