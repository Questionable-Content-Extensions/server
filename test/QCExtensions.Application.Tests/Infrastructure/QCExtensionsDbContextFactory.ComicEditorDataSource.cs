using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Expressions;
using System.Threading;
using System.Threading.Tasks;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Tests.Infrastructure
{
	public partial class QCExtensionsDbContextFactory
	{
		private class ComicEditorDataSource : IAsyncEnumerable<ComicEditorData>, IQueryable<ComicEditorData>
		{
			private class ComicEditorDataAsyncEnumerator : IAsyncEnumerator<ComicEditorData>
			{
				public ComicEditorData Current => null;

				public void Dispose()
				{
				}

				public Task<bool> MoveNext(CancellationToken cancellationToken)
				{
					return Task.FromResult(false);
				}
			}

			private List<ComicEditorData> _source = new List<ComicEditorData>();
			private IQueryable<ComicEditorData> Queryable => _source.AsQueryable();
			public Type ElementType => Queryable.ElementType;

			public Expression Expression => Queryable.Expression;

			public IQueryProvider Provider => Queryable.Provider;

			public IAsyncEnumerator<ComicEditorData> GetEnumerator()
			{
				return new ComicEditorDataAsyncEnumerator();
			}

			IEnumerator<ComicEditorData> IEnumerable<ComicEditorData>.GetEnumerator()
			{
				return _source.GetEnumerator();
			}

			IEnumerator IEnumerable.GetEnumerator()
			{
				return _source.GetEnumerator();
			}
		}
	}
}