use crate::highlight::types::*;
use crate::highlight::matcher::*;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct BoundaryState {
    pub pos_matches: Vec<usize>,
    pub pos_first_match: i32,
    pub pos_last_match: i32,
    pub length_matches_all: usize,
}

#[derive(Debug, Clone)]
pub struct BoundaryTerm {
    pub is_match: bool,
    pub content: String,
    pub original_pos: usize,
}

pub fn apply_advanced_boundary(
    terms: Vec<BoundaryTerm>,
    config: &HighlightConfig,
) -> Result<String> {
    let boundary = match config.boundary.as_ref() {
        Some(b) => b,
        None => return join_boundary_terms(&terms),
    };

    let boundary_total = boundary.total.unwrap_or(900000);
    let boundary_before = boundary.before.unwrap_or(0);
    let boundary_after = boundary.after.unwrap_or(0);

    // Collect match positions
    let match_positions: Vec<usize> = terms
        .iter()
        .enumerate()
        .filter_map(|(i, term)| if term.is_match { Some(i) } else { None })
        .collect();

    if match_positions.is_empty() {
        return join_boundary_terms(&terms);
    }

    let _first_match = match_positions[0];
    let _last_match = match_positions[match_positions.len() - 1];

    // Calculate total match length
    let _total_match_length: usize = terms
        .iter()
        .filter(|term| term.is_match)
        .map(|term| term.content.len())
        .sum();

    let markup_length = match_positions.len() * (config.template.len() - 2);
    let ellipsis_length = if config.ellipsis.is_empty() { 0 } else { config.ellipsis.len() };

    // Check if boundary processing is needed
    if boundary_before == 0 && boundary_after == 0 {
        let total_length: usize = terms.iter().map(|term| term.content.len()).sum::<usize>() + terms.len().saturating_sub(1);
        if total_length - markup_length <= boundary_total {
            return join_boundary_terms(&terms);
        }
    }

    // Advanced boundary processing
    let boundary_length = boundary_total + markup_length - ellipsis_length * 2;
    let match_span = calculate_match_span(&terms, &match_positions);

    let (start_pos, end_pos) = calculate_boundary_range(
        &terms,
        &match_positions,
        match_span,
        boundary_before,
        boundary_after,
        boundary_length,
    );

    apply_boundary_clipping(&terms, start_pos, end_pos, config)
}

fn join_boundary_terms(terms: &[BoundaryTerm]) -> Result<String> {
    let mut result = String::new();
    for (i, term) in terms.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        result.push_str(&term.content);
    }
    Ok(result)
}

fn calculate_match_span(terms: &[BoundaryTerm], match_positions: &[usize]) -> (usize, usize) {
    let first_match = match_positions[0];
    let last_match = match_positions[match_positions.len() - 1];

    let start_len: usize = terms[..first_match]
        .iter()
        .map(|term| term.content.len() + 1) // +1 for space
        .sum::<usize>()
        .saturating_sub(1); // Remove last space

    let match_length: usize = terms[first_match..=last_match]
        .iter()
        .map(|term| term.content.len())
        .sum::<usize>() + (last_match - first_match); // spaces between terms

    (start_len, start_len + match_length)
}

fn calculate_boundary_range(
    terms: &[BoundaryTerm],
    _match_positions: &[usize],
    match_span: (usize, usize),
    boundary_before: i32,
    boundary_after: i32,
    boundary_length: usize,
) -> (usize, usize) {
    let (match_start, match_end) = match_span;
    let match_length = match_end - match_start;

    let start_offset = if boundary_before > 0 {
        boundary_before as usize
    } else {
        (boundary_length - match_length) / 2
    };

    let end_offset = if boundary_after > 0 {
        boundary_after as usize
    } else {
        boundary_length - match_length - start_offset
    };

    let start_pos = match_start.saturating_sub(start_offset);
    let end_pos = usize::min(match_end + end_offset, calculate_total_length(terms));

    (start_pos, end_pos)
}

fn calculate_total_length(terms: &[BoundaryTerm]) -> usize {
    terms
        .iter()
        .map(|term| term.content.len())
        .sum::<usize>() + terms.len().saturating_sub(1)
}

fn apply_boundary_clipping(
    terms: &[BoundaryTerm],
    start_pos: usize,
    end_pos: usize,
    config: &HighlightConfig,
) -> Result<String> {
    let mut result = String::new();
    let mut current_pos = 0;
    let mut in_range = false;
    let need_ellipsis_start = start_pos > 0;
    let need_ellipsis_end = end_pos < calculate_total_length(terms);

    for (i, term) in terms.iter().enumerate() {
        let term_length = term.content.len();
        let term_start = current_pos;
        let term_end = current_pos + term_length;

        // Check if this term overlaps with our range
        if term_end >= start_pos && term_start <= end_pos {
            if !in_range {
                // Starting the range
                if need_ellipsis_start && !config.ellipsis.is_empty() {
                    result.push_str(&config.ellipsis);
                    if !result.is_empty() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                }
                in_range = true;
            }

            // Add the term
            if !result.is_empty() && i > 0 {
                result.push(' ');
            }
            result.push_str(&term.content);
        }

        current_pos = term_end + 1; // +1 for space
    }

    if in_range && need_ellipsis_end && !config.ellipsis.is_empty() {
        if !result.is_empty() && !result.ends_with(' ') {
            result.push(' ');
        }
        result.push_str(&config.ellipsis);
    }

    Ok(result)
}