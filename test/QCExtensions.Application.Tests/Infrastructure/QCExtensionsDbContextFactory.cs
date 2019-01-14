using System;
using Microsoft.EntityFrameworkCore;
using Moq;
using QCExtensions.Domain.Entities;
using QCExtensions.Persistence;

namespace QCExtensions.Application.Tests.Infrastructure
{
	public partial class QCExtensionsDbContextFactory
	{
		public static QCExtensionsDbContext Create()
		{
			var options = new DbContextOptionsBuilder<QCExtensionsDbContext>()
				.UseInMemoryDatabase(Guid.NewGuid().ToString())
				.Options;

			var contextMock = new Mock<QCExtensionsDbContext>(options) { CallBase = true };
			contextMock.Setup(c => c.QueryComicEditorData(It.IsAny<int>())).Returns(new ComicEditorDataSource());
			var context = contextMock.Object;

			context.Database.EnsureCreated();

			context.Comics.AddRange(new[] {
				new Comic { Id = 1, Title = "Comic 1", PublishDate = new DateTime(2000, 1, 1) },
				new Comic { Id = 2, Title = "Comic 2", PublishDate = new DateTime(2000, 2, 2), IsGuestComic = true, IsNonCanon = true },
				new Comic { Id = 3, Title = "Comic 3", PublishDate = new DateTime(2000, 3, 3) },
				new Comic { Id = 4, Title = "Comic 4", PublishDate = new DateTime(2000, 4, 4), IsGuestComic = true, IsNonCanon = true },
				new Comic { Id = 5, Title = "Comic 5", PublishDate = new DateTime(2000, 5, 5) },
			});

			context.News.Add(new News { ComicId = 3, NewsText = "Comic 3 News" });

			context.SaveChanges();

			return context;
		}

		public static void Destroy(QCExtensionsDbContext context)
		{
			context.Database.EnsureDeleted();

			context.Dispose();
		}
	}
}