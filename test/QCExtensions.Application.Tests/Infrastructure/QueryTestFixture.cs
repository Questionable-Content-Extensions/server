using AutoMapper;
using Microsoft.EntityFrameworkCore;
using Moq;
using QCExtensions.Application.Interfaces;
using QCExtensions.Persistence;
using System;
using Xunit;

namespace QCExtensions.Application.Tests.Infrastructure
{
	public class QueryTestFixture : IDisposable
	{
		public QCExtensionsDbContext Context { get; }
		public IMapper Mapper { get; }
		public INewsUpdater NewsUpdater { get; }

		public QueryTestFixture()
		{
			Context = QCExtensionsDbContextFactory.Create();
			Mapper = AutoMapperFactory.Create();

			var newsUpdater = new Mock<INewsUpdater>();
			NewsUpdater = newsUpdater.Object;
		}

		public void Dispose()
		{
			QCExtensionsDbContextFactory.Destroy(Context);
		}
	}

	[CollectionDefinition("Query collection")]
	public class QueryCollection : ICollectionFixture<QueryTestFixture> { }
}
