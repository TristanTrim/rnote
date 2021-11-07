pub mod modifiernode;

mod imp {
    use super::modifiernode::ModifierNode;

    use gtk4::gdk;
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/selectionmodifier.ui")]
    pub struct SelectionModifier {
        #[template_child]
        pub resize_tl: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_tr: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_bl: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_br: TemplateChild<ModifierNode>,
        #[template_child]
        pub translate_node: TemplateChild<gtk4::Box>,
    }

    impl Default for SelectionModifier {
        fn default() -> Self {
            ModifierNode::static_type();

            Self {
                resize_tl: TemplateChild::<ModifierNode>::default(),
                resize_tr: TemplateChild::<ModifierNode>::default(),
                resize_bl: TemplateChild::<ModifierNode>::default(),
                resize_br: TemplateChild::<ModifierNode>::default(),
                translate_node: TemplateChild::<gtk4::Box>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SelectionModifier {
        const NAME: &'static str = "SelectionModifier";
        type Type = super::SelectionModifier;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SelectionModifier {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.resize_tl
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_tr
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_bl
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_br
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.translate_node.set_cursor(
                gdk::Cursor::from_name("grab", gdk::Cursor::from_name("default", None).as_ref())
                    .as_ref(),
            );
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for SelectionModifier {}
}

use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};
use gtk4::{GestureDrag, PropagationPhase};

use crate::{
    ui::appwindow::RnoteAppWindow, ui::selectionmodifier::modifiernode::ModifierNode, utils,
};

glib::wrapper! {
    pub struct SelectionModifier(ObjectSubclass<imp::SelectionModifier>)
        @extends gtk4::Widget;
}

impl Default for SelectionModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionModifier {
    pub const RESIZE_NODE_SIZE: i32 = 22;
    pub const SELECTION_MIN: f64 = 3.0; // Must be >= TRANSLATE_NODE_SIZE_MIN

    pub fn new() -> Self {
        let selection_modifier: Self =
            glib::Object::new(&[]).expect("Failed to create `SelectionModifier`");
        selection_modifier
    }

    pub fn resize_tl(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self).resize_tl.get()
    }

    pub fn resize_tr(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self).resize_tr.get()
    }

    pub fn resize_bl(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self).resize_bl.get()
    }

    pub fn resize_br(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self).resize_br.get()
    }

    pub fn translate_node(&self) -> gtk4::Box {
        imp::SelectionModifier::from_instance(self)
            .translate_node
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        priv_
            .resize_tl
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow  => @default-return None, move |args| {

                    let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;
                    if let Some(selection_bounds) = selection_bounds {
                        let scalefactor = appwindow.canvas().scalefactor();
                        let offset = args[1].get::<utils::BoxedPos>().unwrap();
                        let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                        let new_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                            selection_bounds.mins[0] + offset[0], selection_bounds.mins[1] + offset[1]],
                            na::point![selection_bounds.maxs[0], selection_bounds.maxs[1]]
                        );
                        let min_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                                new_bounds.maxs[0] - Self::SELECTION_MIN,
                                new_bounds.maxs[1] - Self::SELECTION_MIN
                            ],
                            na::point![
                                new_bounds.maxs[0],
                                new_bounds.maxs[1]
                            ]
                        );
                        let new_bounds = utils::aabb_clamp(new_bounds, Some(min_bounds), None);

                        appwindow.canvas().sheet().strokes_state().borrow_mut().resize_selection(new_bounds);

                        obj.queue_resize();
                        appwindow.canvas().queue_draw();
                    }
                    None
                }),
            )
            .unwrap();

        priv_
            .resize_tr
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow => @default-return None, move |args| {

                    let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;
                    if let Some(selection_bounds) = selection_bounds {
                        let scalefactor = appwindow.canvas().scalefactor();
                        let offset = args[1].get::<utils::BoxedPos>().unwrap();
                        let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                        let new_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                            selection_bounds.mins[0], selection_bounds.mins[1] + offset[1]],
                            na::point![selection_bounds.maxs[0] + offset[0], selection_bounds.maxs[1]]
                        );
                        let min_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                                new_bounds.mins[0],
                                new_bounds.maxs[1] - Self::SELECTION_MIN
                            ],
                            na::point![
                                new_bounds.mins[0] + Self::SELECTION_MIN,
                                new_bounds.maxs[1]
                            ]
                        );
                        let new_bounds = utils::aabb_clamp(new_bounds, Some(min_bounds), None);

                        appwindow.canvas().sheet().strokes_state().borrow_mut().resize_selection(new_bounds);

                        obj.queue_resize();
                        appwindow.canvas().queue_draw();
                    }
                    None
                }),
            )
            .unwrap();

        priv_
            .resize_bl
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow => @default-return None, move |args| {

                    let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;
                    if let Some(selection_bounds) = selection_bounds {
                        let scalefactor = appwindow.canvas().scalefactor();
                        let offset = args[1].get::<utils::BoxedPos>().unwrap();
                        let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                        let new_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                            selection_bounds.mins[0] + offset[0], selection_bounds.mins[1]],
                            na::point![selection_bounds.maxs[0], selection_bounds.maxs[1] + offset[1]]
                        );
                        let min_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                                new_bounds.maxs[0] - Self::SELECTION_MIN,
                                new_bounds.mins[1]
                            ],
                            na::point![
                                new_bounds.maxs[0],
                                new_bounds.mins[1] + Self::SELECTION_MIN
                            ]
                        );
                        let new_bounds = utils::aabb_clamp(new_bounds, Some(min_bounds), None);

                        appwindow.canvas().sheet().strokes_state().borrow_mut().resize_selection(new_bounds);

                        obj.queue_resize();
                        appwindow.canvas().queue_draw();
                    }
                    None
                }),
            )
            .unwrap();

        priv_
            .resize_br
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow => @default-return None, move |args| {

                    let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;
                    if let Some(selection_bounds) = selection_bounds {
                        let scalefactor = appwindow.canvas().scalefactor();
                        let offset = args[1].get::<utils::BoxedPos>().unwrap();
                        let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                        let new_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                            selection_bounds.mins[0], selection_bounds.mins[1]],
                            na::point![selection_bounds.maxs[0] + offset[0], selection_bounds.maxs[1] + offset[1]]
                        );
                        let min_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                                new_bounds.mins[0],
                                new_bounds.mins[1]
                            ],
                            na::point![
                                new_bounds.mins[0] + Self::SELECTION_MIN,
                                new_bounds.mins[1] + Self::SELECTION_MIN
                            ]
                        );
                        let new_bounds = utils::aabb_clamp(new_bounds, Some(min_bounds), None);

                        appwindow.canvas().sheet().strokes_state().borrow_mut().resize_selection(new_bounds);

                        obj.queue_resize();
                        appwindow.canvas().queue_draw();
                    }
                    None
                }),
            )
            .unwrap();

        let translate_drag_gesture = GestureDrag::builder()
            .name("translate_drag")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.translate_node.add_controller(&translate_drag_gesture);
        translate_drag_gesture.connect_drag_update(
            clone!(@weak self as obj, @weak appwindow => move |_translate_drag_gesture, x, y| {
                let scalefactor = appwindow.canvas().scalefactor();
                let offset = na::vector![x.round() / scalefactor, y.round() / scalefactor];

                appwindow.canvas().sheet().strokes_state().borrow_mut().translate_selection(offset);

                obj.queue_resize();
                appwindow.canvas().queue_draw();
            }),
        );
    }
}
