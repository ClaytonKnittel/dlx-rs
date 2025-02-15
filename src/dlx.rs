use std::{
  borrow::BorrowMut,
  collections::{HashMap, HashSet},
  fmt::{self, Debug, Formatter},
  hash::Hash,
  iter,
  marker::PhantomData,
};

macro_rules! dlx_unreachable {
  ($msg:expr) => {
    if cfg!(debug_assertions) {
      unreachable!($msg)
    } else {
      unsafe { std::hint::unreachable_unchecked() }
    }
  };
  () => {
    if cfg!(debug_assertions) {
      unreachable!()
    } else {
      unsafe { std::hint::unreachable_unchecked() }
    }
  };
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ColorItem<I> {
  item: I,
  color: u32,
}

impl<I> ColorItem<I> {
  pub fn new(item: I, color: u32) -> Self {
    ColorItem { item, color }
  }

  pub fn item(&self) -> &I {
    &self.item
  }

  pub fn color(&self) -> u32 {
    self.color
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Constraint<I> {
  Primary(I),
  Secondary(ColorItem<I>),
}

impl<I> Constraint<I> {
  fn item(&self) -> &I {
    match self {
      Constraint::Primary(item) | Constraint::Secondary(ColorItem { item, .. }) => item,
    }
  }

  fn color(&self) -> Option<u32> {
    match self {
      Constraint::Primary(_) => None,
      Constraint::Secondary(ColorItem { color, .. }) => Some(*color),
    }
  }
}

impl<I> From<I> for Constraint<I> {
  fn from(value: I) -> Self {
    Constraint::Primary(value)
  }
}

impl<I> From<ColorItem<I>> for Constraint<I> {
  fn from(value: ColorItem<I>) -> Self {
    Constraint::Secondary(value)
  }
}

struct ListNodeI<I> {
  prev: I,
  next: I,
}

type HeaderListNode = ListNodeI<u32>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeaderType {
  Primary,
  Secondary,
}

struct Header<I> {
  item: Option<I>,
  node: HeaderListNode,
  header_type: HeaderType,
}

impl<I> Header<I> {
  fn is_primary(&self) -> bool {
    match self.header_type {
      HeaderType::Primary => true,
      HeaderType::Secondary => false,
    }
  }
}

impl<I> Debug for Header<I>
where
  I: Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{} (prev: {}, next: {}) ({})",
      match &self.item {
        Some(item) => format!("{item:?}"),
        None => "[None]".to_string(),
      },
      self.node.prev,
      self.node.next,
      match self.header_type {
        HeaderType::Primary => "Primary",
        HeaderType::Secondary => "Secondary",
      }
    )
  }
}

type ListNode = ListNodeI<usize>;

enum NodeType {
  Header {
    /// Number of constraints that have this item.
    size: usize,
  },
  Body {
    /// The assigned color of this node, or None if this is a primary constraint.
    color: Option<u32>,
    /// The index of the header node associated with this node.
    top: u32,
  },
}

enum Node<N> {
  Boundary {
    /// The name of the subset listed to the left of this boundary.
    name: Option<N>,
    /// The index of the first node in the subset that comes before this
    /// boundary.
    first_for_prev: usize,
    /// The index of the last node in the subset that comes after this
    /// boundary.
    last_for_next: usize,
  },
  Normal {
    /// Node in linked list of item.
    item_node: ListNode,
    node_type: NodeType,
  },
}

impl<I> Node<I> {
  fn color(&self) -> Option<u32> {
    match self {
      Node::Normal {
        node_type: NodeType::Body { color, .. },
        ..
      } => *color,
      _ => dlx_unreachable!("Unexpected color() called on non-body node"),
    }
  }

  fn color_mut(&mut self) -> &mut Option<u32> {
    match self {
      Node::Normal {
        node_type: NodeType::Body { color, .. },
        ..
      } => color,
      _ => dlx_unreachable!("Unexpected color() called on non-body node"),
    }
  }

  fn len(&self) -> usize {
    match self {
      Node::Normal {
        node_type: NodeType::Header { size },
        ..
      } => *size,
      _ => dlx_unreachable!("Node::len() called on non-header node"),
    }
  }

  fn len_mut(&mut self) -> &mut usize {
    match self {
      Node::Normal {
        node_type: NodeType::Header { size },
        ..
      } => size,
      _ => dlx_unreachable!("Node::len_mut() called on non-header node"),
    }
  }

  fn prev(&self) -> usize {
    match self {
      Node::Normal { item_node, .. } => item_node.prev,
      Node::Boundary { .. } => dlx_unreachable!("Cannot call Node::prev() on a Boundary node"),
    }
  }

  fn set_prev(&mut self, idx: usize) {
    match self {
      Node::Normal { item_node, .. } => item_node.prev = idx,
      Node::Boundary { .. } => dlx_unreachable!("Cannot call Node::set_prev() on a Boundary node"),
    }
  }

  fn next(&self) -> usize {
    match self {
      Node::Normal { item_node, .. } => item_node.next,
      Node::Boundary { .. } => dlx_unreachable!("Cannot call Node::next() on a Boundary node"),
    }
  }

  fn set_next(&mut self, idx: usize) {
    match self {
      Node::Normal { item_node, .. } => item_node.next = idx,
      Node::Boundary { .. } => dlx_unreachable!("Cannot call Node::set_next() on a Boundary node"),
    }
  }
}

impl<N> Debug for Node<N>
where
  N: Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Node::Boundary {
        name,
        first_for_prev,
        last_for_next,
      } => {
        write!(
          f,
          "{}: (first_prev: {}, last_next: {})",
          match name {
            Some(name) => format!("{name:?}"),
            None => "[None]".to_string(),
          },
          first_for_prev,
          last_for_next
        )
      }
      Node::Normal {
        item_node: ListNodeI { prev, next },
        node_type,
      } => {
        write!(
          f,
          "(prev: {}, next: {}) ({})",
          prev,
          next,
          match node_type {
            NodeType::Header { size } => {
              format!("Header (size: {})", size)
            }
            NodeType::Body { color, top } => {
              format!(
                "Body (top: {top}){}",
                match color {
                  Some(color) => format!(" (color: {color})"),
                  None => "".to_string(),
                }
              )
            }
          }
        )
      }
    }
  }
}

enum ChooseNextItemResult {
  Continue,
  FoundSolution,
}

enum ExploreNextChoiceResult {
  Continue,
  Done,
}

pub struct Dlx<I, N> {
  num_primary_items: usize,
  headers: Vec<Header<I>>,
  body: Vec<Node<N>>,
}

impl<I, N> Dlx<I, N> {
  fn header(&self, idx: usize) -> &Header<I> {
    debug_assert!((..self.headers.len()).contains(&idx));
    unsafe { self.headers.get_unchecked(idx) }
  }

  fn header_mut(&mut self, idx: usize) -> &mut Header<I> {
    debug_assert!((..self.headers.len()).contains(&idx));
    unsafe { self.headers.get_unchecked_mut(idx) }
  }

  fn body_header(&self, idx: usize) -> &Node<N> {
    debug_assert!((1..(self.headers.len() - 1)).contains(&idx));
    unsafe { self.body.get_unchecked(idx) }
  }

  fn body_header_mut(&mut self, idx: usize) -> &mut Node<N> {
    debug_assert!((1..(self.headers.len() - 1)).contains(&idx));
    unsafe { self.body.get_unchecked_mut(idx) }
  }

  fn body_node(&self, idx: usize) -> &Node<N> {
    debug_assert!(((self.headers.len() - 1)..self.body.len()).contains(&idx));
    unsafe { self.body.get_unchecked(idx) }
  }

  fn body_node_mut(&mut self, idx: usize) -> &mut Node<N> {
    debug_assert!(((self.headers.len() - 1)..self.body.len()).contains(&idx));
    unsafe { self.body.get_unchecked_mut(idx) }
  }

  fn node(&self, idx: usize) -> &Node<N> {
    debug_assert!(
      (1..(self.headers.len() - 1)).contains(&idx)
        || (self.headers.len()..self.body.len()).contains(&idx)
    );
    unsafe { self.body.get_unchecked(idx) }
  }

  fn node_mut(&mut self, idx: usize) -> &mut Node<N> {
    debug_assert!(
      (1..(self.headers.len() - 1)).contains(&idx)
        || (self.headers.len()..self.body.len()).contains(&idx)
    );
    unsafe { self.body.get_unchecked_mut(idx) }
  }

  fn to_top(&self, mut p: usize) -> usize {
    p = self.body_node(p).next();
    while let Node::Normal {
      node_type,
      item_node,
    } = self.node(p)
    {
      match node_type {
        NodeType::Header { .. } => return p,
        NodeType::Body { .. } => {
          p = item_node.next;
        }
      }
    }
    dlx_unreachable!("Unexpected boundary node found in queue: {p}");
  }

  fn iterate_items(&self, idx: usize) -> impl Iterator<Item = usize> + '_ {
    debug_assert!(matches!(
      self.body_node(idx),
      Node::Normal {
        node_type: NodeType::Body { .. },
        ..
      }
    ));
    iter::repeat(())
      .scan(idx + 1, move |q_ptr, _| {
        let q = *q_ptr;
        if q == idx {
          return None;
        }
        match self.body_node(q) {
          Node::Boundary { first_for_prev, .. } => {
            *q_ptr = *first_for_prev;
            Some(None)
          }
          Node::Normal {
            node_type: NodeType::Body { .. },
            ..
          } => {
            *q_ptr += 1;
            Some(Some(q))
          }
          _ => None,
        }
      })
      .flatten()
  }

  /// Remove the subset containing the node at `idx` from the grid.
  fn hide(&mut self, idx: usize) {
    let mut q = idx.wrapping_add(1);
    while q != idx {
      match self.body_node(q) {
        Node::Boundary { first_for_prev, .. } => {
          q = *first_for_prev;
        }
        Node::Normal {
          item_node,
          node_type: NodeType::Body { top, color },
        } => {
          let top = *top as usize;

          if self.header(top).is_primary() || color.is_some() {
            let prev_idx = item_node.prev;
            let next_idx = item_node.next;
            self.node_mut(prev_idx).set_next(next_idx);
            self.node_mut(next_idx).set_prev(prev_idx);
          }
          let len_mut = self.body_header_mut(top).len_mut();
          *len_mut = len_mut.wrapping_sub(1);
          q = q.wrapping_add(1);
        }
        Node::Normal {
          node_type: NodeType::Header { .. },
          ..
        } => dlx_unreachable!("Unexpected header encountered in hide() at index {q}"),
      }
    }
  }

  /// Reverts `hide(idx)`, assuming the state of Dlx was exactly as it was when
  /// `hide(idx)` was called.
  fn unhide(&mut self, idx: usize) {
    let mut q = idx.wrapping_sub(1);
    while q != idx {
      match self.body_node(q) {
        Node::Boundary { last_for_next, .. } => {
          q = *last_for_next;
        }
        Node::Normal {
          item_node,
          node_type: NodeType::Body { top, color },
        } => {
          let top = *top as usize;

          if self.header(top).is_primary() || color.is_some() {
            let prev_idx = item_node.prev;
            let next_idx = item_node.next;
            self.node_mut(prev_idx).set_next(q);
            self.node_mut(next_idx).set_prev(q);
          }
          let len_mut = self.body_header_mut(top).len_mut();
          *len_mut = len_mut.wrapping_add(1);
          q = q.wrapping_sub(1);
        }
        Node::Normal {
          node_type: NodeType::Header { .. },
          ..
        } => dlx_unreachable!("Unexpected header encountered in unhide() at index {q}"),
      }
    }
  }

  /// Remove all subsets which contain the header item `idx`, and hide the item
  /// from the items list.
  fn cover(&mut self, idx: usize) {
    // println!("Covering {:?}", self.header(idx).item.as_ref().unwrap());
    debug_assert!((1..=self.num_primary_items).contains(&idx));
    let mut p = self.body_header(idx).next();
    while p != idx {
      self.hide(p);
      p = self.body_node(p).next();
    }

    // Hide this item in the items list.
    let header = self.header(idx);
    let prev_idx = header.node.prev;
    let next_idx = header.node.next;
    self.header_mut(prev_idx as usize).node.next = next_idx;
    self.header_mut(next_idx as usize).node.prev = prev_idx;
  }

  /// Reverts `cover(idx)`, assuming the state of Dlx was exactly as it was
  /// when `cover(idx)` was called.
  fn uncover(&mut self, idx: usize) {
    debug_assert!((1..=self.num_primary_items).contains(&idx));
    // Put this item back in the items list.
    let header = self.header(idx);
    let prev_idx = header.node.prev;
    let next_idx = header.node.next;
    self.header_mut(prev_idx as usize).node.next = idx as u32;
    self.header_mut(next_idx as usize).node.prev = idx as u32;

    let mut p = self.body_header(idx).prev();
    while p != idx {
      self.unhide(p);
      p = self.body_node(p).prev();
    }
    // println!("Uncovering {:?}", self.header(idx).item.as_ref().unwrap());
  }

  /// Covers all subsets with secondary constraints which don't have the same
  /// color as the constraint at index `idx`.
  fn purify(&mut self, idx: usize) {
    let (color, top) = match self.body_node(idx) {
      Node::Normal {
        node_type: NodeType::Body {
          color: Some(color),
          top,
        },
        ..
      } => (*color, *top as usize),
      _ => dlx_unreachable!("Unexpected uncolored node for secondary constraint at index {idx}."),
    };
    // println!(
    //   "Purifying {:?} (top {top}, color {})",
    //   self.header(self.to_top(idx)).item.as_ref().unwrap(),
    //   char::from_u32(color).unwrap_or('?')
    // );

    let mut p = self.body_header(top).next();
    while p != top {
      let p_color = self.body_node_mut(p).color_mut();
      // println!("Looking at {p} ({p_color:?})");
      if *p_color == Some(color) {
        *p_color = None;
      } else {
        self.hide(p);
      }
      p = self.body_node(p).next();
    }
  }

  /// Reverts `purify(idx)`, assuming the state of Dlx was exactly as it was
  /// when `purify(idx)` was called.
  fn unpurify(&mut self, idx: usize) {
    let (color, top) = match self.body_node(idx) {
      Node::Normal {
        node_type: NodeType::Body {
          color: Some(color),
          top,
        },
        ..
      } => (*color, *top as usize),
      _ => dlx_unreachable!("Unexpected uncolored node for secondary constraint at index {idx}."),
    };

    let mut p = self.body_header(top).prev();
    while p != top {
      let p_color = self.body_node_mut(p).color_mut();
      if p_color.is_none() {
        *p_color = Some(color);
      } else {
        self.unhide(p);
      }
      p = self.body_node(p).prev();
    }
    // println!(
    //   "Unpurifying {:?}",
    //   self.header(self.to_top(idx)).item.as_ref().unwrap()
    // );
  }

  fn commit(&mut self, idx: usize, top: usize) {
    // println!("Committing {idx} (top: {top})");
    if self.header(top).is_primary() {
      self.cover(top);
    } else if self.body_node(idx).color().is_some() {
      self.purify(idx);
    }
  }

  fn uncommit(&mut self, idx: usize, top: usize) {
    if self.header(top).is_primary() {
      self.uncover(top);
    } else if self.body_node(idx).color().is_some() {
      self.unpurify(idx);
    }
    // println!("Uncommitting {idx} (top: {top})");
  }

  /// Covers all other items take by the subset containing the node at `idx`.
  fn cover_remaining_choices(&mut self, idx: usize) {
    // println!("Covering remaining for {idx}");
    let mut p = idx.wrapping_add(1);
    while p != idx {
      match self.body_node(p) {
        Node::Boundary { first_for_prev, .. } => {
          p = *first_for_prev;
        }
        Node::Normal {
          node_type: NodeType::Body { top, .. },
          ..
        } => {
          self.commit(p, *top as usize);
          p = p.wrapping_add(1);
        }
        Node::Normal {
          node_type: NodeType::Header { .. },
          ..
        } => {
          dlx_unreachable!(
            "Unexpected header encountered in cover_remaining_choices() at index {p}"
          )
        }
      }
    }
  }

  /// Covers all other items take by the subset containing the node at `idx`.
  fn uncover_remaining_choices(&mut self, idx: usize) {
    let mut p = idx.wrapping_sub(1);
    while p != idx {
      match self.body_node(p) {
        Node::Boundary { last_for_next, .. } => {
          p = *last_for_next;
        }
        Node::Normal {
          node_type: NodeType::Body { top, .. },
          ..
        } => {
          self.uncommit(p, *top as usize);
          p = p.wrapping_sub(1);
        }
        Node::Normal {
          node_type: NodeType::Header { .. },
          ..
        } => {
          dlx_unreachable!(
            "Unexpected header encountered in uncover_remaining_choices() at index {p}"
          )
        }
      }
    }
    // println!("Uncovering remaining for {idx}");
  }

  /// Chooses the index of the next item to try covering, using the LRV
  /// heuristic (least remaining values). Returns None if there are no items
  /// left, meaning a solution has been found.
  fn choose_item(&self) -> Option<u32> {
    let mut opt = self.header(0).node.next;
    let mut best_opt = (None, 0);
    while opt != 0 {
      let len = self.body_header(opt as usize).len();
      best_opt = match best_opt {
        (Some(_), min_len) => {
          if min_len > len {
            (Some(opt), len)
          } else {
            best_opt
          }
        }
        (None, _) => (Some(opt), len),
      };

      opt = self.header(opt as usize).node.next;
    }

    best_opt.0
  }

  pub fn find_solutions(&mut self) -> impl DlxIterator<I, N> + '_ {
    DlxIteratorImpl::new(self)
  }

  pub fn into_solutions(self) -> impl DlxIterator<I, N> {
    DlxIteratorImpl::new(self)
  }

  pub fn find_solutions_stepwise(
    &mut self,
  ) -> impl DlxIterator<I, N, StepwiseDlxIterResult<Vec<usize>>> + '_ {
    StepwiseDlxIteratorImpl::new(self)
  }

  pub fn into_solutions_stepwise(
    self,
  ) -> impl DlxIterator<I, N, StepwiseDlxIterResult<Vec<usize>>> {
    StepwiseDlxIteratorImpl::new(self)
  }
}

impl<I, N> Dlx<I, N>
where
  I: Clone,
{
  fn item_name(&self, idx: usize) -> I {
    debug_assert!(matches!(
      self.body_node(idx),
      Node::Normal {
        node_type: NodeType::Body { .. },
        ..
      }
    ));
    if let Node::Normal {
      node_type: NodeType::Body { top, .. },
      ..
    } = self.body_node(idx)
    {
      self.header(*top as usize).item.clone().unwrap()
    } else {
      dlx_unreachable!()
    }
  }

  fn items_for_node(&self, idx: usize) -> impl Iterator<Item = Constraint<I>> + '_ {
    self
      .iterate_items(idx)
      .map(move |item_idx| match self.body_node(item_idx).color() {
        Some(color) => ColorItem::new(self.item_name(item_idx), color).into(),
        None => self.item_name(item_idx).into(),
      })
  }
}

impl<I, N> Dlx<I, N>
where
  N: Clone,
{
  fn set_name_for_node(&self, idx: usize) -> N {
    ((idx + 1)..)
      .find_map(|q| match self.body_node(q) {
        Node::Boundary { name, .. } => Some(name.clone().unwrap()),
        Node::Normal { .. } => None,
      })
      .unwrap()
  }
}

impl<I, N> Dlx<I, N>
where
  I: Hash + Eq + Clone + Debug,
  N: Hash + Eq + Clone + Debug,
{
  pub fn new<U, S, C, D>(items: U, subsets: S) -> Self
  where
    U: IntoIterator<Item = (I, HeaderType)>,
    S: IntoIterator<Item = (N, C)>,
    C: IntoIterator<Item = D>,
    D: Into<Constraint<I>>,
  {
    Self::construct(items, subsets)
  }

  fn construct<U, S, C, D>(items: U, subsets: S) -> Self
  where
    U: IntoIterator<Item = (I, HeaderType)>,
    S: IntoIterator<Item = (N, C)>,
    C: IntoIterator<Item = D>,
    D: Into<Constraint<I>>,
  {
    let mut headers = vec![Header {
      item: None,
      node: ListNodeI { prev: 0, next: 1 },
      header_type: HeaderType::Primary,
    }];
    let mut item_map = HashMap::new();
    let mut body = Vec::new();
    let mut last_start_index;
    let mut subset_names = HashSet::new();

    // Push phony node to first element of body.
    body.push(Node::Boundary {
      name: None,
      first_for_prev: 0,
      last_for_next: 0,
    });

    let (primary_headers, secondary_headers): (Vec<_>, Vec<_>) =
      items
        .into_iter()
        .partition(|(_, header_type)| match header_type {
          HeaderType::Primary => true,
          HeaderType::Secondary => false,
        });

    let primary_headers_len = primary_headers.len() as u32;
    headers.extend(
      primary_headers
        .into_iter()
        .chain(secondary_headers)
        .enumerate()
        .map(|(idx, (item, header_type))| {
          let new_idx = idx + 1;
          if item_map.insert(item.clone(), new_idx).is_some() {
            panic!("Duplicate item {:?}", item);
          }
          body.push(Node::Normal {
            item_node: ListNodeI {
              prev: new_idx,
              next: new_idx,
            },
            node_type: NodeType::Header { size: 0 },
          });

          Header {
            item: Some(item),
            node: ListNodeI {
              prev: new_idx as u32 - 1,
              next: new_idx as u32 + 1,
            },
            header_type,
          }
        }),
    );
    let last_idx = headers.len();
    headers.push(Header {
      item: None,
      node: ListNodeI {
        prev: last_idx as u32 - 1,
        next: primary_headers_len + 1,
      },
      header_type: HeaderType::Secondary,
    });
    headers.get_mut(0).unwrap().node.prev = primary_headers_len;
    headers
      .get_mut(primary_headers_len as usize)
      .unwrap()
      .node
      .next = 0;
    headers
      .get_mut(primary_headers_len as usize + 1)
      .unwrap()
      .node
      .prev = last_idx as u32;
    headers.get_mut(last_idx).unwrap().node.next = primary_headers_len + 1;

    body.push(Node::Boundary {
      name: None,
      first_for_prev: 0,
      last_for_next: 0,
    });

    for (name, constraints) in subsets {
      if !subset_names.insert(name.clone()) {
        panic!("Duplicate subset name: {name:?}");
      }

      last_start_index = body.len();
      constraints.into_iter().for_each(|constraint| {
        let constraint: Constraint<I> = constraint.into();
        let idx = body.len();

        let header_idx = *item_map
          .get(constraint.item())
          .unwrap_or_else(|| panic!("Unknown item {:?}", constraint.item()));
        let header = body.get_mut(header_idx).unwrap();
        let prev_idx = header.prev();

        debug_assert!(
          matches!(
            (headers.get(header_idx).unwrap(), &constraint),
            (
              Header {
                header_type: HeaderType::Primary,
                ..
              },
              Constraint::Primary(_),
            ) | (
              Header {
                header_type: HeaderType::Secondary,
                ..
              },
              Constraint::Secondary(_),
            )
          ),
          "Expect constraint type to match item type (primary vs. secondary)"
        );

        header.set_prev(idx);
        *header.len_mut() += 1;
        body.get_mut(prev_idx).unwrap().set_next(idx);

        body.push(Node::Normal {
          item_node: ListNodeI {
            prev: prev_idx,
            next: header_idx,
          },
          node_type: NodeType::Body {
            color: constraint.color(),
            top: header_idx as u32,
          },
        });
      });

      let last_idx = body.len() - 1;
      if let Some(Node::Boundary { last_for_next, .. }) = body.get_mut(last_start_index - 1) {
        *last_for_next = last_idx;
      } else {
        dlx_unreachable!();
      }

      body.push(Node::Boundary {
        name: Some(name),
        first_for_prev: last_start_index,
        last_for_next: 0,
      });
    }

    let num_primary_items = headers.first().unwrap().node.prev as usize;
    Dlx {
      headers,
      body,
      num_primary_items,
    }
  }
}

impl<I, N> Debug for Dlx<I, N>
where
  I: Debug,
  N: Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    for (idx, header) in self.headers.iter().enumerate() {
      writeln!(f, "{idx:<3} H: {header:?}")?;
    }
    for (idx, node) in self.body.iter().enumerate() {
      writeln!(f, "{idx:<3} N: {:?}", node)?;
    }
    Ok(())
  }
}

enum DlxStepResult<'a> {
  Continue,
  FoundSolution(&'a Vec<usize>),
  Done,
}

#[derive(Debug)]
enum DlxExplorerState {
  Started,
  NotStarted,
}

#[derive(Debug)]
struct DlxExplorer<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  dlx: D,
  partial_solution: Vec<usize>,
  state: DlxExplorerState,
  _phantom: PhantomData<(I, N)>,
}

impl<D, I, N> DlxExplorer<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  fn new(dlx: D) -> Self {
    Self {
      dlx,
      partial_solution: Vec::new(),
      state: DlxExplorerState::NotStarted,
      _phantom: PhantomData,
    }
  }

  fn partial_solution(&self) -> &Vec<usize> {
    &self.partial_solution
  }

  fn dlx(&self) -> &Dlx<I, N> {
    self.dlx.borrow()
  }

  fn dlx_mut(&mut self) -> &mut Dlx<I, N> {
    self.dlx.borrow_mut()
  }

  #[must_use]
  fn choose_next_item(&mut self) -> ChooseNextItemResult {
    let dlx = self.dlx_mut();

    match dlx.choose_item() {
      Some(item) => {
        let item = item as usize;
        dlx.cover(item);
        self.partial_solution.push(item);
        ChooseNextItemResult::Continue
      }
      None => ChooseNextItemResult::FoundSolution,
    }
  }

  #[must_use]
  fn explore_next_choice(&mut self) -> ExploreNextChoiceResult {
    while let Some(p) = self.partial_solution.pop() {
      let dlx = self.dlx_mut();

      if let Node::Normal {
        node_type: NodeType::Body { .. },
        ..
      } = dlx.node(p)
      {
        dlx.uncover_remaining_choices(p);
      }

      // Try exploring the next choice.
      let p = dlx.node(p).next();

      match dlx.node(p) {
        Node::Normal {
          node_type: NodeType::Header { .. },
          ..
        } => {
          // We have exhausted all options under this item, so continue to the
          // previous item.
          dlx.uncover(p);
        }
        Node::Normal {
          node_type: NodeType::Body { .. },
          ..
        } => {
          // We can try exploring this subset.
          dlx.cover_remaining_choices(p);
          self.partial_solution.push(p);
          return ExploreNextChoiceResult::Continue;
        }
        Node::Boundary { .. } => dlx_unreachable!("Unexpected boundary node found in queue: {p}"),
      }
    }

    ExploreNextChoiceResult::Done
  }

  fn step(&mut self) -> DlxStepResult<'_> {
    // This should only be false the very first call to `next()`, or if
    // `next()` is called after `None` is returned at the end of iteration.
    if matches!(self.state, DlxExplorerState::Started) {
      if let ExploreNextChoiceResult::Done = self.explore_next_choice() {
        return DlxStepResult::Done;
      }
    } else {
      self.state = DlxExplorerState::Started;
    }

    if let ChooseNextItemResult::FoundSolution = self.choose_next_item() {
      return DlxStepResult::FoundSolution(&self.partial_solution);
    }

    DlxStepResult::Continue
  }
}

impl<D, I, N> Drop for DlxExplorer<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  fn drop(&mut self) {
    // Undo all changes we've made to the data structure before dropping,
    // leaving it unmodified.
    self.partial_solution.clone().iter().rev().for_each(|&p| {
      let dlx = self.dlx_mut();
      if let Node::Normal {
        node_type: NodeType::Body { .. },
        ..
      } = dlx.node(p)
      {
        dlx.uncover_remaining_choices(p);
        dlx.uncover(dlx.to_top(p));
      } else {
        dlx.uncover(p);
      }
    });
  }
}

pub trait DlxIterator<I, N, R = Vec<usize>>: Iterator<Item = R> + Sized {
  fn dlx(&self) -> &Dlx<I, N>;

  fn mapped<F, S>(self, f: F) -> impl DlxIterator<I, N, S>
  where
    F: FnMut(&Dlx<I, N>, R) -> S,
  {
    MappedDlxIterator::new(self, f)
  }
}

pub trait DlxIteratorWithNames<I, N, R = Vec<N>> {
  fn with_names(self) -> impl DlxIterator<I, N, R>;
}

impl<D, I, N> DlxIteratorWithNames<I, N> for D
where
  D: DlxIterator<I, N, Vec<usize>>,
  N: Clone,
{
  fn with_names(self) -> impl DlxIterator<I, N, Vec<N>> {
    self.mapped(|dlx, solution| {
      solution
        .into_iter()
        .filter_map(|p| {
          if let Node::Normal {
            node_type: NodeType::Body { .. },
            ..
          } = dlx.node(p)
          {
            Some(dlx.set_name_for_node(p))
          } else {
            None
          }
        })
        .collect()
    })
  }
}

impl<D, I, N> DlxIteratorWithNames<I, N, StepwiseDlxIterResult<Vec<N>>> for D
where
  D: DlxIterator<I, N, StepwiseDlxIterResult<Vec<usize>>>,
  N: Clone,
{
  fn with_names(self) -> impl DlxIterator<I, N, StepwiseDlxIterResult<Vec<N>>> {
    self.mapped(|dlx, solution| {
      let is_step = matches!(solution, StepwiseDlxIterResult::Step(_));
      let solution_vec = match solution {
        StepwiseDlxIterResult::Step(solution) | StepwiseDlxIterResult::Solution(solution) => {
          solution
            .into_iter()
            .filter_map(|p| {
              if let Node::Normal {
                node_type: NodeType::Body { .. },
                ..
              } = dlx.node(p)
              {
                Some(dlx.set_name_for_node(p))
              } else {
                None
              }
            })
            .collect()
        }
      };

      if is_step {
        StepwiseDlxIterResult::Step(solution_vec)
      } else {
        StepwiseDlxIterResult::Solution(solution_vec)
      }
    })
  }
}

pub trait DlxIteratorWithColors<I, N> {
  fn with_colors(self) -> impl DlxIterator<I, N, HashMap<I, u32>>;
}

impl<D, I, N> DlxIteratorWithColors<I, N> for D
where
  D: DlxIterator<I, N, Vec<usize>>,
  I: Clone + Eq + Hash,
{
  fn with_colors(self) -> impl DlxIterator<I, N, HashMap<I, u32>> {
    self.mapped(|dlx, solution| {
      solution
        .iter()
        .fold(HashMap::new(), |secondary_assignments, &p| {
          if let Node::Normal {
            node_type: NodeType::Body { .. },
            ..
          } = dlx.node(p)
          {
            dlx
              .items_for_node(p)
              .fold(secondary_assignments, |mut secondary_assignments, c| {
                if let Constraint::Secondary(ColorItem { item, color }) = c {
                  if let Some(prev_color) = secondary_assignments.insert(item, color) {
                    debug_assert_eq!(color, prev_color);
                  }
                }
                secondary_assignments
              })
          } else {
            secondary_assignments
          }
        })
    })
  }
}

#[derive(Debug)]
pub struct DlxIteratorImpl<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  explorer: DlxExplorer<D, I, N>,
}

impl<D, I, N> DlxIteratorImpl<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  fn new(dlx: D) -> Self {
    Self {
      explorer: DlxExplorer::new(dlx),
    }
  }
}

impl<D, I, N> Iterator for DlxIteratorImpl<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  type Item = Vec<usize>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.explorer.step() {
        DlxStepResult::Continue => {}
        DlxStepResult::FoundSolution(solution) => {
          return Some(solution.clone());
        }
        DlxStepResult::Done => return None,
      }
    }
  }
}

impl<D, I, N> DlxIterator<I, N, Vec<usize>> for DlxIteratorImpl<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  fn dlx(&self) -> &Dlx<I, N> {
    self.explorer.dlx()
  }
}

#[derive(Clone, Debug)]
pub enum StepwiseDlxIterResult<T> {
  /// This is a partial solution to the DLX problem.
  Step(T),
  /// This is a complete solution to the DLX problem.
  Solution(T),
}

impl<T> StepwiseDlxIterResult<T> {
  pub fn take_result(self) -> T {
    match self {
      Self::Step(result) | Self::Solution(result) => result,
    }
  }

  pub fn result(&self) -> &T {
    match self {
      Self::Step(result) | Self::Solution(result) => result,
    }
  }
}

#[derive(Debug)]
pub struct StepwiseDlxIteratorImpl<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  explorer: DlxExplorer<D, I, N>,
}

impl<D, I, N> StepwiseDlxIteratorImpl<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  fn new(dlx: D) -> Self {
    Self {
      explorer: DlxExplorer::new(dlx),
    }
  }
}

impl<D, I, N> Iterator for StepwiseDlxIteratorImpl<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  type Item = StepwiseDlxIterResult<Vec<usize>>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.explorer.step() {
      DlxStepResult::Continue => Some(StepwiseDlxIterResult::Step(
        self.explorer.partial_solution().clone(),
      )),
      DlxStepResult::FoundSolution(solution) => {
        Some(StepwiseDlxIterResult::Solution(solution.clone()))
      }
      DlxStepResult::Done => None,
    }
  }
}

impl<D, I, N> DlxIterator<I, N, StepwiseDlxIterResult<Vec<usize>>>
  for StepwiseDlxIteratorImpl<D, I, N>
where
  D: BorrowMut<Dlx<I, N>>,
{
  fn dlx(&self) -> &Dlx<I, N> {
    self.explorer.dlx()
  }
}

#[derive(Debug)]
pub struct MappedDlxIterator<I, N, Iter, R, F, S>
where
  Iter: DlxIterator<I, N, R>,
  F: FnMut(&Dlx<I, N>, R) -> S,
{
  iter: Iter,
  f: F,
  _phony: PhantomData<(I, N, R, S)>,
}

impl<I, N, Iter, R, F, S> MappedDlxIterator<I, N, Iter, R, F, S>
where
  Iter: DlxIterator<I, N, R>,
  F: FnMut(&Dlx<I, N>, R) -> S,
{
  fn new(iter: Iter, f: F) -> Self {
    Self {
      iter,
      f,
      _phony: PhantomData,
    }
  }
}

impl<I, N, Iter, R, F, S> Iterator for MappedDlxIterator<I, N, Iter, R, F, S>
where
  Iter: DlxIterator<I, N, R>,
  F: FnMut(&Dlx<I, N>, R) -> S,
{
  type Item = S;

  fn next(&mut self) -> Option<Self::Item> {
    self
      .iter
      .next()
      .map(|result| (self.f)(self.iter.dlx(), result))
  }
}

impl<I, N, Iter, R, F, S> DlxIterator<I, N, S> for MappedDlxIterator<I, N, Iter, R, F, S>
where
  Iter: DlxIterator<I, N, R>,
  F: FnMut(&Dlx<I, N>, R) -> S,
{
  fn dlx(&self) -> &Dlx<I, N> {
    self.iter.dlx()
  }
}

#[cfg(test)]
mod test {
  use googletest::gtest;
  use itertools::Itertools;

  use googletest::prelude::*;

  use crate::{
    dlx::{ColorItem, Constraint},
    DlxIteratorWithNames, StepwiseDlxIterResult,
  };

  use super::{Dlx, HeaderType};

  #[gtest]
  fn test_empty() {
    let mut dlx: Dlx<u32, u32> = Dlx::new::<_, _, Vec<_>, u32>(vec![], vec![]);

    assert_that!(
      dlx.find_solutions().collect_vec(),
      elements_are![eq(&vec![])]
    );
  }

  #[test]
  fn test_simple() {
    let mut dlx = Dlx::new(vec![(1, HeaderType::Primary)], vec![(0, vec![1])]);

    assert!(dlx
      .find_solutions()
      .with_names()
      .next()
      .is_some_and(|solution| solution.eq(&vec![0])));
  }

  #[test]
  fn test_choose_two() {
    let mut dlx = Dlx::new(
      vec![
        ('p', HeaderType::Primary),
        ('q', HeaderType::Primary),
        ('r', HeaderType::Primary),
      ],
      vec![
        (0, vec!['p', 'q']),
        (1, vec!['p', 'r']),
        (2, vec!['p']),
        (3, vec!['q']),
      ],
    );

    assert!(dlx
      .find_solutions()
      .with_names()
      .next()
      .is_some_and(|mut solution| {
        solution.sort();
        solution.eq(&vec![1, 3])
      }));
  }

  #[test]
  fn test_solve_twice() {
    let mut dlx = Dlx::new(
      vec![
        ('p', HeaderType::Primary),
        ('q', HeaderType::Primary),
        ('r', HeaderType::Primary),
      ],
      vec![
        (0, vec!['p', 'q']),
        (1, vec!['p', 'r']),
        (2, vec!['p']),
        (3, vec!['q']),
      ],
    );

    let solutions1 = dlx.find_solutions().collect_vec();
    let solutions2 = dlx.find_solutions().collect_vec();
    assert_eq!(solutions1, solutions2);
  }

  #[test]
  fn test_solve_partial_twice() {
    let mut dlx = Dlx::new(
      vec![
        ('p', HeaderType::Primary),
        ('q', HeaderType::Primary),
        ('r', HeaderType::Primary),
      ],
      vec![
        (0, vec!['p', 'q']),
        (1, vec!['p', 'r']),
        (2, vec!['p']),
        (3, vec!['q']),
      ],
    );

    let solution1 = dlx.find_solutions().next();
    let solution2 = dlx.find_solutions().next();
    assert_eq!(solution1, solution2);
  }

  #[test]
  fn test_simple_colors() {
    let mut dlx = Dlx::new(
      vec![
        ('p', HeaderType::Primary),
        ('q', HeaderType::Primary),
        ('a', HeaderType::Secondary),
      ],
      vec![
        (
          0,
          vec![Constraint::Primary('p'), ColorItem::new('a', 1).into()],
        ),
        (1, vec!['p'.into(), ColorItem::new('a', 2).into()]),
        (2, vec!['q'.into(), ColorItem::new('a', 3).into()]),
        (3, vec!['q'.into(), ColorItem::new('a', 1).into()]),
      ],
    );

    assert!(dlx
      .find_solutions()
      .with_names()
      .next()
      .is_some_and(|solution| { solution.into_iter().sorted().eq(vec![0, 3].into_iter()) }));
  }

  #[gtest]
  fn test_stepwise() {
    let mut dlx = Dlx::new(
      vec![
        ('p', HeaderType::Primary),
        ('q', HeaderType::Primary),
        ('r', HeaderType::Primary),
      ],
      vec![
        (0, vec!['p', 'q']),
        (1, vec!['p', 'r']),
        (2, vec!['p']),
        (3, vec!['q']),
      ],
    );

    let mut stepwise_iter = dlx.find_solutions_stepwise().with_names();
    assert_that!(
      stepwise_iter.next(),
      some(pat!(StepwiseDlxIterResult::Step(elements_are![])))
    );
    assert_that!(
      stepwise_iter.next(),
      some(pat!(StepwiseDlxIterResult::Step(elements_are![&1])))
    );
    assert_that!(
      stepwise_iter.next(),
      some(pat!(StepwiseDlxIterResult::Solution(elements_are![&1, &3])))
    );
    assert_that!(stepwise_iter.next(), none());
  }

  #[gtest]
  fn test_stepwise_two_solutions() {
    let mut dlx = Dlx::new(
      vec![
        ('p', HeaderType::Primary),
        ('q', HeaderType::Primary),
        ('r', HeaderType::Primary),
      ],
      vec![
        (0, vec!['p', 'q']),
        (1, vec!['p']),
        (2, vec!['p', 'q']),
        (3, vec!['r']),
      ],
    );

    let mut stepwise_iter = dlx.find_solutions_stepwise().with_names();
    assert_that!(
      stepwise_iter.next(),
      some(pat!(StepwiseDlxIterResult::Step(elements_are![])))
    );
    assert_that!(
      stepwise_iter.next(),
      some(pat!(StepwiseDlxIterResult::Step(elements_are![&3])))
    );
    assert_that!(
      stepwise_iter.next(),
      some(pat!(StepwiseDlxIterResult::Solution(elements_are![&3, &0])))
    );
    assert_that!(
      stepwise_iter.next(),
      some(pat!(StepwiseDlxIterResult::Solution(elements_are![&3, &2])))
    );
    assert_that!(stepwise_iter.next(), none());
  }
}
