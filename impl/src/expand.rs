use syn::{ItemFn, parse_quote};
use proc_macro2::{TokenStream, Span};
use quote::ToTokens;
use syn::visit_mut::VisitMut;
use syn::{ExprMethodCall, Ident};
use syn::punctuated::Punctuated;
use syn::{Expr, PathSegment, PathArguments, ExprPath, Stmt, Path};

#[derive(Default)]
struct DependencyVisitor {
    query: Option<Box<Expr>>,
}

impl VisitMut for DependencyVisitor {
    fn visit_expr_method_call_mut(&mut self, node: &mut ExprMethodCall) {

        // Detect if the method call is present
        // append this method with another method call.

        if node.method == "execute" {

            // We have to update the parent receiver of this expression
            //
            let updated_parent_receiver = {
                let mut segments = Punctuated::new();
                segments.push(PathSegment {ident: Ident::new("query", Span::call_site()), arguments: PathArguments::None} );
                let expr_path = ExprPath { attrs: Vec::new(), qself: None, path: Path {leading_colon: None, segments}};

                    Expr::Path(expr_path)
            };

            // This expression is the original 'sqlx::query("...")'
            let query = node.receiver.clone();
            // This modifies the 'execute(..)' receiver to be our new ident capturing 'query' into
            // a local let.
            node.receiver = Box::new(updated_parent_receiver);

            // Construct the "dependency" part, which has the "execute" as its receiver.
            let dep = ExprMethodCall {
                attrs: Vec::new(),
                receiver: Box::new(syn::Expr::MethodCall(node.clone())),
                dot_token: node.dot_token.clone(),
                method: Ident::new("dependency", Span::call_site()),
                turbofish: None,
                paren_token: node.paren_token.clone(),
                args: Punctuated::new(),
            };

            let _ = std::mem::replace(node, dep);
            self.query = Some(query);
        }
    }
}

pub fn dependency_fn(item: &mut ItemFn) -> TokenStream {
    for mut statement in &mut item.block.stmts {
        let mut visitor = DependencyVisitor::default();

        match statement {
            Stmt::Expr(expr, _) => {
                visitor.visit_expr_mut(expr);
                
                if let Some(ref query) = visitor.query {
                    let mut block: Stmt = parse_quote!(
                        {
                            let query = #query;
                            let sql = query.sql();
                            #expr
                        }
                    );

                    std::mem::swap(&mut block, &mut statement);
                }
            },
            Stmt::Local(local) => {
                //visitor.visit_expr_mut(expr);
            },
            _ => {}
        }
    }
    
    item.to_token_stream()
}
