use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use tide::Route;

#[allow(clippy::type_complexity)]
#[allow(missing_debug_implementations)]
pub struct VariadicRoutes<State>
where
    State: Send + Sync + 'static,
{
    _phantom_state: PhantomData<State>,
    pub routes: Vec<Box<dyn for<'r> Fn(Route<'r, Arc<State>>)>>,
}

impl<State, RoutesFn> From<RoutesFn> for VariadicRoutes<State>
where
    State: Send + Sync + 'static,
    RoutesFn: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
{
    fn from(routes: RoutesFn) -> Self {
        VariadicRoutes {
            _phantom_state: PhantomData,
            routes: vec![Box::new(routes)],
        }
    }
}

// For completeness only
impl<State, RoutesFn> From<(RoutesFn,)> for VariadicRoutes<State>
where
    State: Send + Sync + 'static,
    RoutesFn: for<'r> Fn(Route<'r, Arc<State>>) + Debug + 'static,
{
    fn from(routes: (RoutesFn,)) -> Self {
        VariadicRoutes {
            _phantom_state: PhantomData,
            routes: vec![Box::new(routes.0)],
        }
    }
}

impl<State, RoutesFn1, RoutesFn2> From<(RoutesFn1, RoutesFn2)> for VariadicRoutes<State>
where
    State: Send + Sync + 'static,
    RoutesFn1: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
    RoutesFn2: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
{
    fn from(routes: (RoutesFn1, RoutesFn2)) -> Self {
        VariadicRoutes {
            _phantom_state: PhantomData,
            routes: vec![Box::new(routes.0), Box::new(routes.1)],
        }
    }
}

impl<State, RoutesFn1, RoutesFn2, RoutesFn3> From<(RoutesFn1, RoutesFn2, RoutesFn3)>
    for VariadicRoutes<State>
where
    State: Send + Sync + 'static,
    RoutesFn1: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
    RoutesFn2: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
    RoutesFn3: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
{
    fn from(routes: (RoutesFn1, RoutesFn2, RoutesFn3)) -> Self {
        VariadicRoutes {
            _phantom_state: PhantomData,
            routes: vec![Box::new(routes.0), Box::new(routes.1), Box::new(routes.2)],
        }
    }
}

impl<State, RoutesFn1, RoutesFn2, RoutesFn3, RoutesFn4>
    From<(RoutesFn1, RoutesFn2, RoutesFn3, RoutesFn4)> for VariadicRoutes<State>
where
    State: Send + Sync + 'static,
    RoutesFn1: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
    RoutesFn2: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
    RoutesFn3: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
    RoutesFn4: for<'r> Fn(Route<'r, Arc<State>>) + 'static,
{
    fn from(routes: (RoutesFn1, RoutesFn2, RoutesFn3, RoutesFn4)) -> Self {
        VariadicRoutes {
            _phantom_state: PhantomData,
            routes: vec![
                Box::new(routes.0),
                Box::new(routes.1),
                Box::new(routes.2),
                Box::new(routes.3),
            ],
        }
    }
}

// If you have api versioning beyond ... I don't know... 5?? you probably should reconsider your architecture!!
#[allow(clippy::type_complexity)]
impl<State> From<Vec<Box<dyn for<'r> Fn(Route<'r, Arc<State>>)>>> for VariadicRoutes<State>
where
    State: Send + Sync + 'static,
{
    fn from(routes: Vec<Box<dyn for<'r> Fn(Route<'r, Arc<State>>)>>) -> Self {
        VariadicRoutes {
            _phantom_state: PhantomData,
            routes,
        }
    }
}
