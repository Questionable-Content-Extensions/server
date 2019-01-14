using System.Collections.Generic;
using QCExtensions.Application.Interfaces;

namespace QCExtensions.Server.Infrastructure.Services
{

	public class NewsUpdater : INewsUpdater
	{
		private HashSet<int> _updateSet = new HashSet<int>();
		private object _updateSetLock = new object();

		public void CheckFor(int comic)
		{
			lock (_updateSetLock)
			{
				_updateSet.Add(comic);
			}
		}

		public ICollection<int> GetPendingUpdateEntries()
		{
			lock (_updateSetLock)
			{
				var updateEntries = new int[_updateSet.Count];
				_updateSet.CopyTo(updateEntries);
				return updateEntries;
			}
		}

		public void RemovePendingUpdateEntries(ICollection<int> updateEntries)
		{
			lock (_updateSetLock)
			{
				_updateSet.RemoveWhere(f => updateEntries.Contains(f));
			}
		}
	}
}
