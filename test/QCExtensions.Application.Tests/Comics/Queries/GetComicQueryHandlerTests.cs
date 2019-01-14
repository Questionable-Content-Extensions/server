using AutoMapper;
using Moq;
using QCExtensions.Application.Comics.Models;
using QCExtensions.Application.Comics.Queries.GetComic;
using QCExtensions.Application.Interfaces;
using QCExtensions.Application.Tests.Infrastructure;
using QCExtensions.Domain.Entities;
using Shouldly;
using System.Threading;
using System.Threading.Tasks;
using Xunit;

using static QCExtensions.Application.Comics.Queries.GetComic.GetComicQuery;

namespace QCExtensions.Application.Tests.Comics.Queries
{
	[Collection("Query collection")]
	public class GetComicQueryHandlerTests
	{
		private readonly DomainDbContext _context;
		private readonly IMapper _mapper;
		private readonly INewsUpdater _newsUpdater;

		public GetComicQueryHandlerTests(QueryTestFixture fixture)
		{
			_context = fixture.Context;
			_mapper = fixture.Mapper;
			_newsUpdater = fixture.NewsUpdater;
		}

		[Fact]
		public async Task NonExistentComicHasDataIsFalse()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 1000 }, CancellationToken.None);

			result.ShouldBeOfType<ComicDto>();
			result.Comic.ShouldBe(1000);
			result.HasData.ShouldBe(false);
		}

		[Fact]
		public async Task ExistentComicHasDataIsTrue()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 3 }, CancellationToken.None);

			result.ShouldBeOfType<ComicDto>();
			result.Comic.ShouldBe(3);
			result.HasData.ShouldBe(true);
		}

		[Fact]
		public async Task PreviousIsNullWhenComicIsFirstComic()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 1 }, CancellationToken.None);

			result.Previous.ShouldBeNull();
		}

		[Fact]
		public async Task NextIsNullWhenComicIsLastComic()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 5 }, CancellationToken.None);

			result.Next.ShouldBeNull();
		}

		[Fact]
		public async Task PreviousAndNextAreSet()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 3 }, CancellationToken.None);

			result.Previous.ShouldBe(2);
			result.Next.ShouldBe(4);
		}

		[Fact]
		public async Task PreviousAndNextShouldSkipGuestComicsWhenExcludeIsSetToGuest()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 3, Exclude = Exclusion.Guest }, CancellationToken.None);

			result.Previous.ShouldBe(1);
			result.Next.ShouldBe(5);
		}

		[Fact]
		public async Task PreviousAndNextShouldSkipNonCanonComicsWhenExcludeIsSetToNonCanon()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 3, Exclude = Exclusion.NonCanon }, CancellationToken.None);

			result.Previous.ShouldBe(1);
			result.Next.ShouldBe(5);
		}

		[Fact]
		public async Task NewsUpdaterShouldBeCalledIfComicExists() {
			var newsUpdater = new Mock<INewsUpdater>();
			var sut = new GetComicQueryHandler(_context, _mapper, newsUpdater.Object);
			await sut.Handle(new GetComicQuery { ComicId = 3 }, CancellationToken.None);

			newsUpdater.Verify(n => n.CheckFor(3));
		}

		[Fact]
		public async Task NewsUpdaterShouldNotBeCalledIfComicDoesNotExist() {
			var newsUpdater = new Mock<INewsUpdater>();
			var sut = new GetComicQueryHandler(_context, _mapper, newsUpdater.Object);
			await sut.Handle(new GetComicQuery { ComicId = 5, Exclude = Exclusion.NonCanon }, CancellationToken.None);

			newsUpdater.Verify(n => n.CheckFor(It.IsAny<int>()), Times.Never());
		}

		[Fact]
		public async Task ComicWithNoNewsHasNullNews()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 5 }, CancellationToken.None);

			result.News.ShouldBeNull();
		}

		[Fact]
		public async Task ComicWithNewsHasNewsText()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 3 }, CancellationToken.None);

			result.News.ShouldNotBeNullOrEmpty();
		}

		[Fact]
		public async Task InvalidTokenHasNoEditorData()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 3 }, CancellationToken.None);

			result.EditorData.ShouldBeNull();
		}

		[Fact]
		public async Task ValidTokenHasEditorData()
		{
			var sut = new GetComicQueryHandler(_context, _mapper, _newsUpdater);
			var result = await sut.Handle(new GetComicQuery { ComicId = 3, IsValidToken = true }, CancellationToken.None);

			result.EditorData.ShouldNotBeNull();
		}
	}
}