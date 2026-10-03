use crate::tokens::ItemStr;

/// Split comment text on `\n`, `\r\n` and `\r`, so that every line gets its
/// own comment prefix instead of leaking into the surrounding code.
pub(crate) fn for_each_line(text: ItemStr, mut f: impl FnMut(ItemStr)) {
    if !text.contains(['\n', '\r']) {
        f(text);
        return;
    }

    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);

        for line in line.split('\r') {
            f(ItemStr::from(line));
        }
    }
}
