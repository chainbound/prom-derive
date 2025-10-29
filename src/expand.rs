use darling::{FromField, FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Field, Ident, ItemStruct, LitFloat, LitStr, Result, Type};

static METRIC_ATTR_NAME: &str = "metric";

#[derive(FromMeta, Debug)]
#[darling(derive_syn_parse)]
pub(super) struct MetricsAttr {
    /// The scope to use for the metrics. Used as a prefix for metric names.
    scope: Option<LitStr>,
}

impl MetricsAttr {
    /// The default separator to use between the scope and the metric name.
    /// NOTE: Prometheus does not support `.` as a separator.
    const DEFAULT_SEPARATOR: &'static str = "_";

    /// Get the separator.
    fn separator(&self) -> String {
        Self::DEFAULT_SEPARATOR.to_string()
    }
}

#[derive(Debug, Clone, Copy)]
enum PrometheusMetricType {
    /// [prometheus::IntCounter].
    IntCounter,
    /// [prometheus::Counter].
    Counter,
    /// [prometheus::IntGauge].
    IntGauge,
    /// [prometheus::Gauge].
    Gauge,
    /// [prometheus::Histogram].
    Histogram,
}

impl PrometheusMetricType {
    fn from_type(ty: &Type) -> Result<Self> {
        let Type::Path(path) = ty else {
            return Err(syn::Error::new_spanned(ty, "Expected a path type"));
        };

        let last = path.path.segments.last().unwrap();
        let last_ident = &last.ident;

        match last_ident.to_string().as_str() {
            "IntCounter" => Ok(Self::IntCounter),
            "Counter" => Ok(Self::Counter),
            "IntGauge" => Ok(Self::IntGauge),
            "Gauge" => Ok(Self::Gauge),
            "Histogram" => Ok(Self::Histogram),
            _ => Err(syn::Error::new_spanned(
                ty,
                "Expected a valid Prometheus metric type",
            )),
        }
    }

    fn singular_type(&self) -> Type {
        match self {
            Self::IntCounter => syn::parse_quote!(prometheus::IntCounter),
            Self::Counter => syn::parse_quote!(prometheus::Counter),
            Self::IntGauge => syn::parse_quote!(prometheus::IntGauge),
            Self::Gauge => syn::parse_quote!(prometheus::Gauge),
            Self::Histogram => syn::parse_quote!(prometheus::Histogram),
        }
    }

    fn vec_type(&self) -> Type {
        match self {
            Self::IntCounter => syn::parse_quote!(prometheus::IntCounterVec),
            Self::Counter => syn::parse_quote!(prometheus::CounterVec),
            Self::IntGauge => syn::parse_quote!(prometheus::IntGaugeVec),
            Self::Gauge => syn::parse_quote!(prometheus::GaugeVec),
            Self::Histogram => syn::parse_quote!(prometheus::HistogramVec),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::IntCounter => "IntCounter",
            Self::Counter => "Counter",
            Self::IntGauge => "IntGauge",
            Self::Gauge => "Gauge",
            Self::Histogram => "Histogram",
        }
    }
}

#[derive(FromField)]
#[darling(attributes(metric))]
#[allow(dead_code)]
struct MetricField {
    /// The identifier of the field.
    ident: Option<Ident>,
    /// The type of the field.
    ty: Type,
    /// The name override to use for the metric.
    rename: Option<String>,
    /// The label keys to define for the metric.
    labels: Option<Vec<LitStr>>,
    /// The buckets to use for the histogram.
    /// TODO: Implement this.
    buckets: Option<Vec<LitFloat>>,
    /// The sample rate to use for the histogram.
    /// TODO: Implement this.
    sample: Option<LitFloat>,
    #[darling(skip)]
    metric_name: String,
    #[darling(skip)]
    metric_type: Option<PrometheusMetricType>,
    #[darling(skip)]
    doc: Option<String>,
}

impl MetricField {
    fn try_from(field: &Field, scope: &str, separator: &str) -> Result<Self> {
        let doc = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("doc"))
            .map(|attr| {
                let syn::Meta::NameValue(value) = &attr.meta else {
                    return Err(syn::Error::new_spanned(attr, "Expected a doc attribute"));
                };

                if let syn::Expr::Lit(lit) = &value.value {
                    if let syn::Lit::Str(lit) = &lit.lit {
                        Ok(lit.value().trim().to_string())
                    } else {
                        Err(syn::Error::new_spanned(attr, "Expected a string literal"))
                    }
                } else {
                    Err(syn::Error::new_spanned(attr, "Expected a string literal"))
                }
            })
            .transpose()?;

        let mut this = Self::from_field(field)?;

        this.doc = doc;

        let metric_name = this
            .rename
            .as_ref()
            .unwrap_or(&field.ident.as_ref().unwrap().to_string())
            .to_owned();

        let full_name = format!("{}{}{}", scope, separator, metric_name);

        this.metric_name = full_name;

        let ty = PrometheusMetricType::from_type(&this.ty)?;
        this.metric_type = Some(ty);

        this.ty = if this
            .labels
            .as_ref()
            .is_some_and(|labels| !labels.is_empty())
        {
            ty.vec_type()
        } else {
            ty.singular_type()
        };

        Ok(this)
    }

    fn identifier(&self) -> Ident {
        self.ident.clone().unwrap()
    }

    fn labels(&self) -> Vec<String> {
        self.labels
            .as_ref()
            .map(|labels| labels.iter().map(|label| label.value()).collect::<Vec<_>>())
            .unwrap_or_default()
    }

    fn storage_type(&self) -> Type {
        if self.labels().is_empty() {
            self.metric_type().unwrap().singular_type()
        } else {
            self.metric_type().unwrap().vec_type()
        }
    }

    fn metric_type(&self) -> Option<PrometheusMetricType> {
        self.metric_type
    }

    fn doc(&self) -> String {
        self.doc.clone().unwrap_or_default()
    }
}

/// Build the initializer for a metric field.
fn build_initializer(field: &MetricField) -> TokenStream {
    let ident = field.identifier();
    let name = &field.metric_name;
    let labels = field.labels();
    let doc = field.doc();

    let metric_type = field.metric_type().expect("Metric type should be set");

    let ty = if labels.is_empty() {
        metric_type.singular_type()
    } else {
        metric_type.vec_type()
    };

    match metric_type {
        PrometheusMetricType::IntCounter
        | PrometheusMetricType::Counter
        | PrometheusMetricType::IntGauge
        | PrometheusMetricType::Gauge => {
            quote! {
                #ident: {
                    let opts = prometheus::Opts::new(#name, #doc).const_labels(self.labels.clone());
                    let metric = <#ty>::new(opts, &[#(#labels),*]).unwrap();

                    self.registry.register(Box::new(metric.clone())).unwrap();
                    metric
                }
            }
        }
        PrometheusMetricType::Histogram => {
            // TODO: Implement buckets and sample rate.
            quote! {
                #ident: {
                    let opts = prometheus::HistogramOpts::new(#name, #doc).const_labels(self.labels.clone());
                    let metric = <#ty>::new(opts, &[#(#labels),*]).unwrap();

                    self.registry.register(Box::new(metric.clone())).unwrap();
                    metric
                }
            }
        }
    }
}

pub fn snake_to_pascal(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '_' {
            // underscore → mark next char for capitalization, skip underscore
            capitalize_next = true;
        } else if ch.is_ascii_alphanumeric() {
            if capitalize_next {
                // uppercase the char
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                // just push it (lowercase or original)
                result.push(ch.to_ascii_lowercase());
            }
        } else {
            // any other char (dash, space, punctuation) — treat as word-separator
            capitalize_next = true;
        }
    }

    result
}

fn build_accessor(field: &MetricField, vis: &syn::Visibility) -> (TokenStream, TokenStream) {
    let ident = field.identifier();
    let doc = field.doc();
    let labels = field.labels();
    let ty = field.storage_type();
    let metric_type = field.metric_type().expect("Metric type should be set");

    let accessor_name = format_ident!("{}Accessor", snake_to_pascal(&ident.to_string()));

    let label_definitions = labels.iter().map(|label| {
        let label_ident = format_ident!("{label}");
        quote! { #label_ident: Option<String> }
    });

    let label_initializers = labels.iter().map(|label| {
        let label_ident = format_ident!("{label}");

        quote! { #label_ident: None }
    });

    let def_doc = format!("Accessor for the `{}` metric.", ident);
    let definition = quote! {
        #[doc = #def_doc]
        #vis struct #accessor_name<'a> {
            inner: &'a #ty,
            #(#label_definitions),*
        }
    };

    let accessor_doc = format!(
        "{doc}\n\
        * Metric type: {}\n\
        * Labels: {}",
        metric_type.as_str(),
        labels.join(", ")
    );
    let accessor = quote! {
        #[doc = #accessor_doc]
        #[must_use = "This doesn't do anything unless the metric value is changed"]
        #vis fn #ident(&self) -> #accessor_name {
            #accessor_name {
                inner: &self.#ident,
                #(#label_initializers),*
            }
        }

    };

    (definition, accessor)
}

/// Build the implementation for an accessor struct based on the field.
fn build_accessor_impl(field: &MetricField, vis: &syn::Visibility) -> TokenStream {
    let labels = field.labels();
    let ident = field.identifier();

    let accessor_name = format_ident!("{}Accessor", snake_to_pascal(&ident.to_string()));

    // Generate the methods for setting the labels.
    let label_methods = labels.iter().map(|label| {
        let label_ident = format_ident!("{label}");
        quote! {
            #[must_use = "This doesn't do anything unless the metric value is changed"]
            #vis fn #label_ident(mut self, value: impl Into<String>) -> Self {
                self.#label_ident = Some(value.into());
                self
            }
        }
    });

    let label_idents = labels
        .iter()
        .map(|label| format_ident!("{label}"))
        .collect::<Vec<_>>();

    let metric_type = field.metric_type().expect("Metric type should be set");
    let terminal_methods = match metric_type {
        PrometheusMetricType::Histogram => quote! {
            #vis fn observe(&self, value: f64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).observe(value);
            }
        },
        PrometheusMetricType::IntCounter => quote! {
            #vis fn inc(&self) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).inc();
            }

            #vis fn inc_by(&self, value: u64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).inc_by(value);
            }

            #vis fn reset(&self) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).reset();
            }
        },
        PrometheusMetricType::Counter => quote! {
            #vis fn inc(&self) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).inc();
            }

            #vis fn inc_by(&self, value: f64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).inc_by(value);
            }

            #vis fn reset(&self) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).reset();
            }
        },
        PrometheusMetricType::IntGauge => quote! {
            #vis fn inc(&self) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).inc();
            }

            #vis fn dec(&self) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).dec();
            }

            #vis fn sub(&self, value: i64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).sub(value);
            }

            #vis fn set(&self, value: i64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).set(value);
            }

            #vis fn add(&self, value: i64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).add(value);
            }
        },
        PrometheusMetricType::Gauge => quote! {
            #vis fn inc(&self) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).inc();
            }

            #vis fn dec(&self) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).dec();
            }

            #vis fn sub(&self, value: f64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).sub(value);
            }

            #vis fn set(&self, value: f64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).set(value);
            }

            #vis fn add(&self, value: f64) {
                let labels = [#(self.#label_idents.as_deref().unwrap_or("")),*];
                self.inner.with_label_values(&labels).add(value);
            }
        },
    };

    quote! {
        impl<'a> #accessor_name<'a> {
            #(#label_methods)*

            #terminal_methods
        }
    }
}

pub fn expand(metrics_attr: MetricsAttr, input: &mut ItemStruct) -> Result<TokenStream> {
    let mut fields = Vec::with_capacity(input.fields.len());
    let mut initializers = Vec::with_capacity(input.fields.len());
    let mut definitions = Vec::with_capacity(input.fields.len());
    let mut accessor_impls = Vec::with_capacity(input.fields.len());
    let mut accessors = Vec::with_capacity(input.fields.len());

    // The visibility of the metrics struct
    let vis = &input.vis;
    // The identifier of the metrics struct
    let ident = &input.ident;

    for field in input.fields.iter_mut() {
        let metric_field = MetricField::try_from(
            field,
            &metrics_attr.scope.as_ref().unwrap().value(),
            &metrics_attr.separator(),
        )?;

        field.ty = metric_field.ty.clone();

        initializers.push(build_initializer(&metric_field));
        let (definition, accessor) = build_accessor(&metric_field, &vis);
        definitions.push(definition);
        accessors.push(accessor);
        accessor_impls.push(build_accessor_impl(&metric_field, &vis));

        fields.push(metric_field);

        // Remove the metric attribute from the field.
        field
            .attrs
            .retain(|attr| !attr.path().is_ident(METRIC_ATTR_NAME));
    }

    let builder_name = format_ident!("{ident}Builder");

    let mut output = quote! {
        #vis struct #builder_name<'a> {
            registry: &'a prometheus::Registry,
            labels: std::collections::HashMap<String, String>,
        }

        impl<'a> #builder_name<'a> {
            /// Set the registry to use for the metrics.
            #vis fn with_registry(mut self, registry: &'a prometheus::Registry) -> Self {
                self.registry = registry;
                self
            }

            /// Add a static label to the metrics struct.
            #vis fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
                self.labels.insert(key.into(), value.into());
                self
            }

            /// Build and register the metrics with the registry.
            #vis fn build(self) -> #ident {
                #ident {
                    #(#initializers),*
                }
            }
        }

        #input
    };

    output = quote! {
        #output

        impl Default for #ident {
            fn default() -> Self {
                Self::builder().build()
            }
        }

        #(#definitions)*

        #(#accessor_impls)*

        impl #ident {
            /// Create a new builder for the metrics struct.
            /// It will be initialized with the default registry and no labels.
            #vis fn builder<'a>() -> #builder_name<'a> {
                #builder_name {
                    registry: prometheus::default_registry(),
                    labels: std::collections::HashMap::new(),
                }
            }

            #(#accessors)*
        }
    };

    Ok(output)
}
