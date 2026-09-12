use crate::{
    Context, Error, database, types::GraphLayout, types::Phase, utilities::ensure_host_role,
};
use petgraph::Graph;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::{CreateAttachment, CreateMessage};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command as TokioCommand;
#[poise::command(prefix_command, slash_command)]
pub async fn reveal_graph(
    ctx: Context<'_>,
    #[description = "Graph layout"] mut layout: GraphLayout,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    }
    if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {
        return Ok(());
    }
    let users = match database::get_matching_order() {
        Ok(users) => users,
        Err(e) => {
            eprintln!("Error getting matching order: {}", e);

            ctx.send(
                CreateReply::default()
                    .content("Error getting matching order.")
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }
    };
    layout = match layout {
        GraphLayout::Default => GraphLayout::Circo,
        GraphLayout::Random => {
            let layouts = [
                GraphLayout::Dot,
                GraphLayout::Neato,
                GraphLayout::Fdp,
                GraphLayout::Circo,
                GraphLayout::Twopi,
                GraphLayout::Osage,
                GraphLayout::Patchwork,
            ];
            layouts[rand::random_range(0..layouts.len())]
        }
        _ => layout,
    };
    let mut graph = Graph::<&str, &str>::new();
    let mut user_nodes: Vec<NodeIndex> = Vec::new();
    let mut edges: Vec<(NodeIndex, NodeIndex)> = Vec::new();
    // create node on graph then store in user_nodes
    users.iter().for_each(|user| {
        user_nodes.push(graph.add_node(user.1.as_str()));
    });
    //create edges
    for i in 0..user_nodes.len() {
        edges.push((user_nodes[i], user_nodes[(i + 1) % user_nodes.len()]));
    }
    // push edges to graph
    graph.extend_with_edges(&edges);
    let dot_output = format!("{}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    let message = format!("heres the graph :happy: ({:?})", layout);

    let mut graphviz_process = match TokioCommand::new(format!("{:?}", layout).to_lowercase())
        .arg("-Tpng")
        .arg("-Nshape=none")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(process) => process,
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("Command failed, feature may not be available")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };
    let mut graphviz_stdin = graphviz_process.stdin.take().unwrap();
    if let Err(e) = graphviz_stdin.write_all(dot_output.as_bytes()).await {
        eprintln!("Error: {:?}", e);
        ctx.send(
            CreateReply::default()
                .content("Failed to execute command")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };
    drop(graphviz_stdin);
    let graph_data = match graphviz_process.wait_with_output().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            ctx.send(
                CreateReply::default()
                    .content("Command failed")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };
    if !graph_data.status.success() {
        ctx.send(
            CreateReply::default()
                .content("Command failed")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }
    let attachment = CreateAttachment::bytes(graph_data.stdout, "graph.png");
    match ctx
        .author()
        .direct_message(
            ctx.http(),
            CreateMessage::new().content(message).add_file(attachment),
        )
        .await
    {
        Ok(_) => {
            ctx.send(
                CreateReply::default()
                    .content("The graph(s) have been sent to your DMs.")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => {
            eprintln!("Error sending reveal DM: {}", e);

            ctx.send(
                CreateReply::default()
                    .content("I couldn't send you a DM. Please make sure your DMs are open.")
                    .ephemeral(true),
            )
            .await?;
        }
    }
    Ok(())
}