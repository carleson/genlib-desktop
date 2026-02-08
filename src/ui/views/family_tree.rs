//! Familjeträd-vy för visualisering av släktrelationer

use egui::{self, Color32, Pos2, Rect, RichText, Stroke, Vec2};

use crate::db::Database;
use crate::services::{FamilyTree, FamilyTreeService, LinkType};
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
    View,
};

/// Vy för att visa familjeträd
pub struct FamilyTreeView {
    /// Cachat träd
    tree: Option<FamilyTree>,
    /// Person som trädet är centrerat kring
    focus_person_id: Option<i64>,
    /// Antal generationer att visa
    generations: i32,
    /// Pan offset
    pan_offset: Vec2,
    /// Zoom level
    zoom: f32,
    /// Behöver refresh
    needs_refresh: bool,
}

impl Default for FamilyTreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl FamilyTreeView {
    pub fn new() -> Self {
        Self {
            tree: None,
            focus_person_id: None,
            generations: 3,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            needs_refresh: true,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        // Kontrollera om vi har en person att visa
        let person_id = state.selected_person_id;

        if person_id.is_none() {
            self.show_no_person_selected(ui, state);
            return;
        }

        let person_id = person_id.unwrap();

        // Refresh om nödvändigt
        if self.needs_refresh || self.focus_person_id != Some(person_id) {
            self.refresh(db, person_id);
        }

        // Header
        self.show_header(ui, state, db, person_id);

        ui.separator();

        // Trädvy
        if let Some(tree) = &self.tree {
            if tree.nodes.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(RichText::new("Inga relationer att visa").color(Colors::TEXT_MUTED));
                    ui.add_space(20.0);
                    if ui.button("Lägg till relationer").clicked() {
                        state.navigate_to_person(person_id);
                    }
                });
            } else {
                self.show_tree_canvas(ui, state, tree.clone());
            }
        }
    }

    fn show_no_person_selected(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            if ui.button(format!("{} Tillbaka", Icons::ARROW_LEFT)).clicked() {
                state.navigate(View::PersonList);
            }
            ui.heading(format!("{} Familjeträd", Icons::TREE));
        });

        ui.separator();

        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(RichText::new("Ingen person vald").color(Colors::TEXT_MUTED));
            ui.add_space(20.0);
            if ui.button("Välj en person").clicked() {
                state.navigate(View::PersonList);
            }
        });
    }

    fn show_header(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database, person_id: i64) {
        ui.horizontal(|ui| {
            if ui.button(format!("{} Tillbaka", Icons::ARROW_LEFT)).clicked() {
                state.navigate_to_person(person_id);
            }

            ui.separator();

            ui.heading(format!("{} Familjeträd", Icons::TREE));

            // Visa fokuspersonens namn
            if let Ok(Some(person)) = db.persons().find_by_id(person_id) {
                ui.label(format!("- {}", person.full_name()));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Zoom-kontroller
                if ui.button("➕").on_hover_text("Zooma in").clicked() {
                    self.zoom = (self.zoom * 1.2).min(3.0);
                }
                ui.label(format!("{:.0}%", self.zoom * 100.0));
                if ui.button("➖").on_hover_text("Zooma ut").clicked() {
                    self.zoom = (self.zoom / 1.2).max(0.3);
                }

                ui.separator();

                // Generationer
                ui.label("Generationer:");
                if ui.add(egui::DragValue::new(&mut self.generations).range(1..=5)).changed() {
                    self.needs_refresh = true;
                }

                ui.separator();

                // Reset-knapp
                if ui.button("Återställ vy").clicked() {
                    self.pan_offset = Vec2::ZERO;
                    self.zoom = 1.0;
                }
            });
        });
    }

    fn show_tree_canvas(&mut self, ui: &mut egui::Ui, state: &mut AppState, tree: FamilyTree) {
        // Hämta tillgängligt utrymme
        let available_size = ui.available_size();

        // Skapa en scrollarea / canvas
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        let rect = response.rect;
        let center = rect.center();

        // Hantera pan (drag)
        if response.dragged() {
            self.pan_offset += response.drag_delta();
        }

        // Hantera zoom med scroll
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_factor = 1.0 + scroll_delta * 0.001;
                self.zoom = (self.zoom * zoom_factor).clamp(0.3, 3.0);
            }
        }

        // Rita bakgrund
        painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

        // Beräkna transformation
        let transform = |pos: Pos2| -> Pos2 {
            Pos2::new(
                center.x + (pos.x * self.zoom) + self.pan_offset.x,
                center.y + (pos.y * self.zoom) + self.pan_offset.y,
            )
        };

        // Rita länkar först (under noderna)
        for link in &tree.links {
            if let (Some(from_node), Some(to_node)) = (
                tree.nodes.get(&link.from_id),
                tree.nodes.get(&link.to_id),
            ) {
                let from_pos = transform(Pos2::new(from_node.x, from_node.y));
                let to_pos = transform(Pos2::new(to_node.x, to_node.y));

                let (color, thickness) = match link.link_type {
                    LinkType::Parent => (Colors::TEXT_SECONDARY, 2.0),
                    LinkType::Spouse => (Colors::SPOUSE, 2.0),
                    LinkType::Sibling => (Colors::SIBLING, 1.0),
                };

                // Rita linje
                if link.link_type == LinkType::Parent {
                    // Vertikal linje med böj för förälder-barn
                    let mid_y = (from_pos.y + to_pos.y) / 2.0;
                    painter.line_segment(
                        [from_pos, Pos2::new(from_pos.x, mid_y)],
                        Stroke::new(thickness * self.zoom, color),
                    );
                    painter.line_segment(
                        [Pos2::new(from_pos.x, mid_y), Pos2::new(to_pos.x, mid_y)],
                        Stroke::new(thickness * self.zoom, color),
                    );
                    painter.line_segment(
                        [Pos2::new(to_pos.x, mid_y), to_pos],
                        Stroke::new(thickness * self.zoom, color),
                    );
                } else {
                    // Rak linje för partners
                    painter.line_segment(
                        [from_pos, to_pos],
                        Stroke::new(thickness * self.zoom, color),
                    );
                }
            }
        }

        // Rita noder
        let node_width = 140.0 * self.zoom;
        let node_height = 60.0 * self.zoom;
        let mut clicked_person: Option<i64> = None;
        let mut double_clicked_person: Option<i64> = None;

        for (person_id, node) in &tree.nodes {
            let pos = transform(Pos2::new(node.x, node.y));

            let node_rect = Rect::from_center_size(pos, Vec2::new(node_width, node_height));

            // Kolla om noden är inom synlig rect
            if !rect.intersects(node_rect) {
                continue;
            }

            // Bakgrundsfärg baserat på om det är fokuspersonen
            let is_focus = tree.focus_person_id == Some(*person_id);
            let bg_color = if is_focus {
                Colors::PRIMARY
            } else {
                ui.visuals().widgets.inactive.bg_fill
            };

            let text_color = if is_focus {
                Color32::WHITE
            } else {
                ui.visuals().text_color()
            };

            // Rita nod-bakgrund
            painter.rect_filled(node_rect, 8.0 * self.zoom, bg_color);
            painter.rect_stroke(
                node_rect,
                8.0 * self.zoom,
                Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color),
            );

            // Rita namn
            let name = node.person.full_name();
            let font_size = 14.0 * self.zoom;
            painter.text(
                pos + Vec2::new(0.0, -8.0 * self.zoom),
                egui::Align2::CENTER_CENTER,
                &name,
                egui::FontId::proportional(font_size),
                text_color,
            );

            // Rita årtal om tillgängligt
            let years = self.format_years(&node.person);
            if !years.is_empty() {
                let small_font = 11.0 * self.zoom;
                painter.text(
                    pos + Vec2::new(0.0, 10.0 * self.zoom),
                    egui::Align2::CENTER_CENTER,
                    &years,
                    egui::FontId::proportional(small_font),
                    if is_focus { Color32::from_white_alpha(200) } else { Colors::TEXT_MUTED },
                );
            }

            // Kontrollera dubbelklick på nod
            if response.double_clicked() {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    if node_rect.contains(click_pos) {
                        double_clicked_person = Some(*person_id);
                    }
                }
            }

            // Kontrollera enkelklick på nod
            if response.clicked() {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    if node_rect.contains(click_pos) {
                        clicked_person = Some(*person_id);
                    }
                }
            }
        }

        // Hantera dubbelklick - navigera till persondetalj
        if let Some(pid) = double_clicked_person {
            state.navigate_to_person(pid);
        } else if let Some(pid) = clicked_person {
            // Hantera enkelklick - byt fokusperson
            if tree.focus_person_id != Some(pid) {
                self.focus_person_id = Some(pid);
                self.needs_refresh = true;
                self.pan_offset = Vec2::ZERO;
            }
        }

        // Rita instruktioner
        let instructions = "Dra för att panorera • Scrolla för att zooma • Klicka på person för att fokusera • Dubbelklicka för detaljer";
        painter.text(
            Pos2::new(rect.center().x, rect.bottom() - 20.0),
            egui::Align2::CENTER_CENTER,
            instructions,
            egui::FontId::proportional(11.0),
            Colors::TEXT_MUTED,
        );
    }

    fn format_years(&self, person: &crate::models::Person) -> String {
        match (person.birth_date, person.death_date) {
            (Some(b), Some(d)) => format!("{} - {}", b.format("%Y"), d.format("%Y")),
            (Some(b), None) => format!("f. {}", b.format("%Y")),
            (None, Some(d)) => format!("d. {}", d.format("%Y")),
            (None, None) => String::new(),
        }
    }

    fn refresh(&mut self, db: &Database, person_id: i64) {
        self.focus_person_id = Some(person_id);

        let service = FamilyTreeService::new(db);
        match service.build_tree(person_id, self.generations) {
            Ok(tree) => self.tree = Some(tree),
            Err(_) => self.tree = None,
        }

        self.needs_refresh = false;
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }

    pub fn set_person(&mut self, person_id: i64) {
        if self.focus_person_id != Some(person_id) {
            self.focus_person_id = Some(person_id);
            self.needs_refresh = true;
            self.pan_offset = Vec2::ZERO;
        }
    }
}
