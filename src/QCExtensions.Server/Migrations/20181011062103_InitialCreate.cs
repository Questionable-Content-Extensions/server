using System;
using Microsoft.EntityFrameworkCore.Metadata;
using Microsoft.EntityFrameworkCore.Migrations;

namespace QCExtensions.Server.Migrations
{
    public partial class InitialCreate : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "items",
                columns: table => new
                {
                    id = table.Column<int>(nullable: false)
                        .Annotation("MySql:ValueGenerationStrategy", MySqlValueGenerationStrategy.IdentityColumn),
                    shortName = table.Column<string>(maxLength: 50, nullable: false),
                    name = table.Column<string>(maxLength: 255, nullable: false),
                    type = table.Column<string>(maxLength: 255, nullable: false),
                    color = table.Column<string>(maxLength: 6, nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_items", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "news",
                columns: table => new
                {
                    comic = table.Column<int>(nullable: false)
                        .Annotation("MySql:ValueGenerationStrategy", MySqlValueGenerationStrategy.IdentityColumn),
                    lastUpdated = table.Column<DateTime>(nullable: false),
                    news = table.Column<string>(nullable: false),
                    updateFactor = table.Column<double>(nullable: false),
                    isLocked = table.Column<bool>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_news", x => x.comic);
                });

            migrationBuilder.CreateTable(
                name: "token",
                columns: table => new
                {
                    id = table.Column<Guid>(nullable: false),
                    Identifier = table.Column<string>(maxLength: 50, nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_token", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "comic",
                columns: table => new
                {
                    id = table.Column<int>(nullable: false),
                    isGuestComic = table.Column<bool>(nullable: false),
                    isNonCanon = table.Column<bool>(nullable: false),
                    title = table.Column<string>(maxLength: 255, nullable: false),
                    tagline = table.Column<string>(maxLength: 255, nullable: true),
                    publishDate = table.Column<DateTime>(nullable: true),
                    isAccuratePublishDate = table.Column<bool>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_comic", x => x.id);
                    table.ForeignKey(
                        name: "FK_comic_news_id",
                        column: x => x.id,
                        principalTable: "news",
                        principalColumn: "comic",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "log_entry",
                columns: table => new
                {
                    id = table.Column<int>(nullable: false)
                        .Annotation("MySql:ValueGenerationStrategy", MySqlValueGenerationStrategy.IdentityColumn),
                    UserToken = table.Column<Guid>(nullable: false),
                    DateTime = table.Column<DateTime>(nullable: false),
                    Action = table.Column<string>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_log_entry", x => x.id);
                    table.ForeignKey(
                        name: "FK_log_entry_token_UserToken",
                        column: x => x.UserToken,
                        principalTable: "token",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "occurences",
                columns: table => new
                {
                    comic_id = table.Column<int>(nullable: false),
                    items_id = table.Column<int>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_occurences", x => new { x.comic_id, x.items_id });
                    table.ForeignKey(
                        name: "FK_occurences_comic_comic_id",
                        column: x => x.comic_id,
                        principalTable: "comic",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "FK_occurences_items_items_id",
                        column: x => x.items_id,
                        principalTable: "items",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "IX_log_entry_UserToken",
                table: "log_entry",
                column: "UserToken");

            migrationBuilder.CreateIndex(
                name: "IX_occurences_items_id",
                table: "occurences",
                column: "items_id");
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "log_entry");

            migrationBuilder.DropTable(
                name: "occurences");

            migrationBuilder.DropTable(
                name: "token");

            migrationBuilder.DropTable(
                name: "comic");

            migrationBuilder.DropTable(
                name: "items");

            migrationBuilder.DropTable(
                name: "news");
        }
    }
}
