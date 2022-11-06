use super::errors::ParseError;

#[derive(Clone, Debug, Copy)]
pub struct HttpRange {
    pub start: u64,
    pub length: u64,
}

static PREFIX: &str = "bytes=";
const PREFIX_LEN: usize = 6;

impl HttpRange {
    pub fn parse(header: &str, size: u64) -> Result<Vec<HttpRange>, ParseError> {
        if header.is_empty() {
            return Ok(Vec::new());
        }

        if !header.starts_with(PREFIX) {
            return Err(ParseError::InvalidRange);
        }

        let size_sig = size as i64;
        let mut no_overlap = false;

        let all_ranges: Vec<Option<HttpRange>> = header[PREFIX_LEN..]
            .split(',')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|ra| {
                let mut start_end_iter = ra.split('-');

                let start_str = start_end_iter.next().ok_or(())?.trim();
                let end_str = start_end_iter.next().ok_or(())?.trim();

                if start_str.is_empty() {
                    let mut length: i64 = end_str.parse().map_err(|_| ())?;

                    if length > size_sig {
                        length = size_sig;
                    }

                    Ok(Some(HttpRange {
                        start: (size_sig - length) as u64,
                        length: length as u64,
                    }))
                } else {
                    let start: i64 = start_str.parse().map_err(|_| ())?;
                    if start < 0 {
                        return Err(());
                    }
                    if start >= size_sig {
                        no_overlap = true;
                        return Ok(None);
                    }

                    let length = if end_str.is_empty() {
                        size_sig - start
                    } else {
                        let mut end: i64 = end_str.parse().map_err(|_| ())?;
                        if start > end {
                            return Err(());
                        }

                        if end >= size_sig {
                            end = size_sig - 1;
                        }

                        end - start + 1
                    };

                    Ok(Some(HttpRange {
                        start: start as u64,
                        length: length as u64,
                    }))
                }
            })
            .collect::<Result<_, _>>()
            .map_err(|_| ParseError::InvalidRange)?;

        let ranges: Vec<HttpRange> = all_ranges.into_iter().flatten().collect();
        if no_overlap && ranges.is_empty() {
            return Err(ParseError::InvalidRange);
        }
        Ok(ranges)
    }
}
