use std::cmp::Ordering;

use lazy_static::lazy_static;

use crate::domain::{Highlight, HighlightCategory, Moisture, Plant, Shade};

lazy_static! {
    static ref POLLINATOR_GREAT: Highlight = Highlight {
        label: "Great for pollinators".to_string(),
        priority: 1003,
        category: HighlightCategory::Great
    };
    static ref POLLINATOR_GOOD: Highlight = Highlight {
        label: "Good for pollinators".to_string(),
        priority: 503,
        category: HighlightCategory::Good
    };
    static ref BIRD_GREAT: Highlight = Highlight {
        label: "Great for birds".to_string(),
        priority: 1002,
        category: HighlightCategory::Great
    };
    static ref BIRD_GOOD: Highlight = Highlight {
        label: "Good for birds".to_string(),
        priority: 502,
        category: HighlightCategory::Good
    };
    static ref ANIMAL_GREAT: Highlight = Highlight {
        label: "Great for animals".to_string(),
        priority: 1001,
        category: HighlightCategory::Great
    };
    static ref ANIMAL_GOOD: Highlight = Highlight {
        label: "Good for animals".to_string(),
        priority: 501,
        category: HighlightCategory::Good
    };
    static ref VERY_DEER_RESISTANT: Highlight = Highlight {
        label: "Deer resistant".to_string(),
        priority: 2000,
        category: HighlightCategory::Great
    };
    static ref DEER_RESISTANT: Highlight = Highlight {
        label: "Mildly deer resistant".to_string(),
        priority: 900,
        category: HighlightCategory::Good
    };
    static ref AGGRESSIVE_SPREAD: Highlight = Highlight {
        label: "Spreads aggressively".to_string(),
        priority: 3000,
        category: HighlightCategory::Bad
    };
    static ref VERY_AGGRESSIVE_SPREAD: Highlight = Highlight {
        label: "Spreads aggressively".to_string(),
        priority: 3000,
        category: HighlightCategory::Worse
    };
    static ref GROWS_IN_SHADE: Highlight = Highlight {
        label: "Grows in shade".to_string(),
        priority: 3,
        category: HighlightCategory::Good
    };
    static ref GROWS_IN_PART_SHADE: Highlight = Highlight {
        label: "Grows in partial shade".to_string(),
        priority: 2,
        category: HighlightCategory::Good
    };
    static ref GROWS_IN_DRY_SOIL: Highlight = Highlight {
        label: "Grows in dry soil".to_string(),
        priority: 1,
        category: HighlightCategory::Good
    };
}

impl Ord for HighlightCategory {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl PartialOrd for HighlightCategory {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HighlightCategory {
    fn priority(&self) -> i32 {
        match *self {
            HighlightCategory::Great => 4,
            HighlightCategory::Good => 3,
            HighlightCategory::Bad => 2,
            HighlightCategory::Worse => 1,
        }
    }
}

pub fn generate(plant: &Plant) -> Vec<Highlight> {
    let mut highlights = list_highlights(plant);

    // If there are no main highlights, try to generate some fillers.
    if highlights.is_empty() {
        highlights.extend(list_fillers(plant));
    }

    // Sort the list by priority, with highest priority first.
    highlights.sort_by(|lhs, rhs| rhs.priority.cmp(&lhs.priority));

    // Keep the top three.
    highlights.truncate(3);

    // Sort by category (good/bad) then priority to move bad items to the end
    highlights.sort_by(|lhs, rhs| {
        let category_order = rhs.category.cmp(&lhs.category);
        if category_order == Ordering::Equal {
            rhs.priority.cmp(&lhs.priority)
        } else {
            category_order
        }
    });

    highlights
}

fn list_highlights(plant: &Plant) -> Vec<Highlight> {
    let mut highlights = vec![];

    match &plant.pollinator_rating {
        Some(rating) if rating.rating >= 8 => highlights.push(POLLINATOR_GREAT.clone()),
        Some(rating) if rating.rating >= 6 => highlights.push(POLLINATOR_GOOD.clone()),
        _ => (),
    }
    match &plant.bird_rating {
        Some(rating) if rating.rating >= 8 => highlights.push(BIRD_GREAT.clone()),
        Some(rating) if rating.rating >= 6 => highlights.push(BIRD_GOOD.clone()),
        _ => (),
    }
    match plant.spread_rating {
        Some(rating) if rating >= 8 => highlights.push(VERY_AGGRESSIVE_SPREAD.clone()),
        Some(rating) if rating >= 6 => highlights.push(AGGRESSIVE_SPREAD.clone()),
        _ => (),
    }
    match plant.deer_resistance_rating {
        Some(rating) if rating >= 8 => highlights.push(VERY_DEER_RESISTANT.clone()),
        Some(rating) if rating >= 6 => highlights.push(DEER_RESISTANT.clone()),
        _ => (),
    }

    highlights
}

fn list_fillers(plant: &Plant) -> Vec<Highlight> {
    let mut fillers = vec![];

    if plant.shades.contains(&Shade::Lots) {
        fillers.push(GROWS_IN_SHADE.clone())
    } else if plant.shades.contains(&Shade::Some) {
        fillers.push(GROWS_IN_PART_SHADE.clone())
    }

    if plant.moistures.contains(&Moisture::None) {
        fillers.push(GROWS_IN_DRY_SOIL.clone())
    }

    fillers
}
