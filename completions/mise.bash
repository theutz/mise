_mise() {
    if ! command -v usage &> /dev/null; then
        echo >&2
        echo "Error: usage CLI not found. This is required for completions to work in mise." >&2
        echo "See https://usage.jdx.dev for more information." >&2
        return 1
    fi

    if [[ -z ${_usage_spec_mise_2024_11_16:-} ]]; then
        _usage_spec_mise_2024_11_16="$(mise usage)"
    fi

    COMPREPLY=( $(usage complete-word --shell bash -s "${_usage_spec_mise_2024_11_16}" --cword="$COMP_CWORD" -- "${COMP_WORDS[@]}" ) )
    if [[ $? -ne 0 ]]; then
        unset COMPREPLY
    fi
    return 0
}

shopt -u hostcomplete && complete -o nospace -o bashdefault -o nosort -F _mise mise
# vim: noet ci pi sts=0 sw=4 ts=4 ft=sh
